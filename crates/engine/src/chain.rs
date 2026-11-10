// Ports of processor.cpp's chain machinery: events, triggers, the Execute units that run script functions, AddChain,
// SolveChain and SolveContinuous.
use crate::card::{CardId, CardSet};
use crate::constants::*;
use crate::duel::Duel;
use crate::effect::{Chain, Container, CounterFilter, EffectId, EffectSet, Event, Operation, Target, TriggeringState, Value};
use crate::processor::{PointEvent, RESTART, SelectEffectYesNo, SelectOption};
use crate::script::{Args, Procedure};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecuteKind {
    Cost,
    Target,
    Operation,
}

// ExecuteCost, ExecuteTarget and ExecuteOperation in one unit; `procedure` holds a summon procedure's parameters.
#[derive(Debug, Clone)]
pub struct Execute {
    pub step: u16,
    pub kind: ExecuteKind,
    pub triggering_effect: EffectId,
    pub triggering_player: u8,
    pub procedure: Option<Box<Procedure>>,
}

#[derive(Debug, Clone, Default)]
pub struct AddChain {
    pub step: u16,
    pub is_activated_effect: bool,
}

#[derive(Debug, Clone, Default)]
pub struct SolveChain {
    pub step: u16,
    pub skip_trigger: bool,
    pub skip_freechain: bool,
    pub skip_new: bool,
    pub backed_up_operation: Option<Operation>,
}

#[derive(Debug, Clone, Default)]
pub struct SolveContinuous {
    pub step: u16,
    pub reason_player: u8,
    pub reason_effect: Option<EffectId>,
}

impl Duel {
    pub(crate) fn field_raise_event(
        &mut self,
        event_cards: Option<CardSet>,
        event_code: u32,
        reason_effect: Option<EffectId>,
        reason: u32,
        reason_player: u8,
        event_player: u8,
        event_value: u32,
    ) {
        let global_id = self.infos.event_id;
        self.core.queue_event.push(Event {
            trigger_card: None,
            event_cards,
            reason_effect,
            event_code,
            event_value,
            reason,
            event_player,
            reason_player,
            global_id,
        });
    }

    // The core also takes event cards here; the events that carry some come with the cards that raise them.
    pub(crate) fn field_raise_single_event(
        &mut self,
        trigger_card: CardId,
        event_code: u32,
        reason_effect: Option<EffectId>,
        reason: u32,
        reason_player: u8,
        event_player: u8,
        event_value: u32,
    ) {
        self.raise_single_event_with(
            trigger_card,
            None,
            event_code,
            reason_effect,
            reason,
            reason_player,
            event_player,
            event_value,
        );
    }

    // raise_single_event with the event's card group.
    pub(crate) fn raise_single_event_with(
        &mut self,
        trigger_card: CardId,
        event_cards: Option<CardSet>,
        event_code: u32,
        reason_effect: Option<EffectId>,
        reason: u32,
        reason_player: u8,
        event_player: u8,
        event_value: u32,
    ) {
        let global_id = self.infos.event_id;
        self.core.single_event.push(Event {
            trigger_card: Some(trigger_card),
            event_cards,
            reason_effect,
            event_code,
            event_value,
            reason,
            event_player,
            reason_player,
            global_id,
        });
    }

    pub(crate) fn field_check_event(&self, code: u32) -> Option<Event> {
        self.core
            .point_event
            .iter()
            .chain(&self.core.instant_event)
            .find(|event| event.event_code == code)
            .copied()
    }

    // chain::set_triggering_state.
    pub(crate) fn set_triggering_state(&self, chain: &mut Chain, card: CardId) {
        let current = self.cards[card].current;
        chain.triggering_controler = current.controler;
        chain.triggering_location = u16::from(current.location);
        if current.location & LOCATION_ONFIELD != 0 {
            chain.triggering_location |= [LOCATION_FZONE, LOCATION_PZONE, LOCATION_STZONE, LOCATION_EMZONE, LOCATION_MMZONE]
                .into_iter()
                .find(|&zone| current.is_location(zone))
                .unwrap_or(0);
        }
        chain.triggering_sequence = current.sequence;
        chain.triggering_position = current.position as u8;
        chain.triggering_status = self.cards[card].status;
        chain.triggering_summon_type = self.cards[card].summon.summon_type & 0xff00_ffff;
        chain.triggering_summon_location = self.cards[card].summon.location;
        chain.triggering_summon_proc_complete = self.cards[card].is_status(STATUS_PROC_COMPLETE);
        chain.triggering_state = TriggeringState {
            code: self.get_code(card),
            code2: self.get_another_code(card),
            card_type: self.get_type(card),
            level: self.get_level(card),
            rank: self.get_rank(card),
            attribute: self.get_attribute(card),
            race: self.get_race(card),
            attack: self.get_attack(card),
            defense: self.get_defense(card),
            setcodes: self.get_set_card(card),
        };
    }

    // A new chain link for an effect activated in a chain window (the core repeats this block per collection loop).
    pub(crate) fn chain_candidate(&mut self, effect: EffectId, event: Event, player: u8) -> Chain {
        let mut chain = Chain::new(effect);
        chain.chain_id = self.next_field_id() as u16;
        chain.evt = event;
        let handler = self.get_handler(effect);
        self.set_triggering_state(&mut chain, handler);
        chain.triggering_player = player;
        chain
    }

    pub(crate) fn process_instant_event(&mut self) {
        if self.core.queue_event.is_empty() {
            return;
        }
        let (mut tp, mut ntp) = (
            std::mem::take(&mut self.core.continuous_tp),
            std::mem::take(&mut self.core.continuous_ntp),
        );
        let mut index = 0;
        while let Some(event) = self.core.queue_event.get(index).copied() {
            index += 1;
            for effect in self.field_effect_range(Container::Continuous, event.event_code) {
                let mut owner_player = self.get_handler_player(effect);
                if self.effects[effect].is_flag(EFFECT_FLAG_EVENT_PLAYER) && event.event_player < 2 {
                    owner_player = event.event_player;
                }
                if !self.is_activateable(effect, owner_player, &event, false, false, false, false, false) {
                    continue;
                }
                let chain = Chain {
                    triggering_effect: effect,
                    triggering_player: owner_player,
                    evt: event,
                    target_player: PLAYER_NONE,
                    disable_player: PLAYER_NONE,
                    ..Chain::default()
                };
                self.continuous_chain_list(effect, owner_player, &mut tp, &mut ntp).push(chain);
            }
            if event.event_code == EVENT_ADJUST
                || event.event_code == EVENT_BREAK_EFFECT
                || event.event_code & 0xffff_f000 == EVENT_PHASE_START
            {
                continue;
            }
            for effect in self.field_effect_range(Container::TriggerF, event.event_code) {
                let handler = self.get_handler(effect);
                let controler = self.cards[handler].current.controler;
                if !self.cards[handler].is_status(STATUS_EFFECT_ENABLED) || !self.is_condition_check(effect, controler, &event) {
                    continue;
                }
                let was_just_sent = self.check_simultaneous(effect, handler);
                if was_just_sent && self.effects[effect].range & u16::from(LOCATION_HAND) == 0 {
                    continue;
                }
                let chain = self.new_trigger(effect, &event, was_just_sent);
                self.create_relation(handler, effect, chain.chain_id);
                self.core.new_fchain.push(chain);
            }
            for effect in self.field_effect_range(Container::TriggerO, event.event_code) {
                let handler = self.get_handler(effect);
                let controler = self.cards[handler].current.controler;
                let act = self.cards[handler].is_status(STATUS_EFFECT_ENABLED) && self.is_condition_check(effect, controler, &event);
                let was_just_sent = self.check_simultaneous(effect, handler);
                let range = self.effects[effect].range;
                if (was_just_sent || !act) && range & u16::from(LOCATION_HAND) == 0 {
                    continue;
                }
                let chain = self.new_trigger(effect, &event, was_just_sent);
                let location = u16::from(self.cards[handler].current.location);
                if self.effects[effect].is_flag(EFFECT_FLAG_FIELD_ONLY)
                    || range & u16::from(LOCATION_HAND) == 0
                    || (range & location != 0 && act)
                {
                    self.create_relation(handler, effect, chain.chain_id);
                }
                self.core.new_ochain.push(chain);
            }
            for effect in self.field_effect_range(Container::QuickF, event.event_code) {
                let handler = self.get_handler(effect);
                self.set_activate_location(effect);
                let controler = self.cards[handler].current.controler;
                if self.is_activateable(effect, controler, &event, false, false, false, false, false) {
                    self.insert_quick_f_chain(effect, &event, handler);
                }
            }
            self.core.delayed_activate_event.push(event);
            for effect in self.field_effect_range(Container::QuickO, event.event_code) {
                let controler = self.cards[self.get_handler(effect)].current.controler;
                if self.effects[effect].is_flag(EFFECT_FLAG_DELAY) && self.is_condition_check(effect, controler, &event) {
                    self.insert_delayed_quick(effect, event);
                }
            }
        }
        self.push_continuous(tp, ntp);
        let core = &mut self.core;
        core.instant_event.append(&mut core.queue_event);
    }

    // core.quick_f_chain[effect], a map ordered by effect address (the effect id stands in for it): an entry the effect
    // already has is overwritten.
    fn insert_quick_f_chain(&mut self, effect: EffectId, event: &Event, handler: CardId) {
        let existing = self.core.quick_f_chain.iter().position(|&(other, _)| other == effect);
        let mut chain = existing
            .and_then(|index| self.core.quick_f_chain.get(index))
            .map_or_else(|| Chain::new(effect), |entry| entry.1);
        chain.flag = 0;
        chain.chain_id = self.next_field_id() as u16;
        chain.evt = *event;
        chain.triggering_effect = effect;
        self.set_triggering_state(&mut chain, handler);
        chain.triggering_player = if self.effects[effect].is_flag(EFFECT_FLAG_EVENT_PLAYER) && event.event_player < 2 {
            event.event_player
        } else {
            self.cards[handler].current.controler
        };
        self.create_relation(handler, effect, chain.chain_id);
        match existing {
            Some(index) => {
                if let Some(entry) = self.core.quick_f_chain.get_mut(index) {
                    entry.1 = chain;
                }
            }
            None => {
                let id = self.effects.id(effect);
                let index = self.core.quick_f_chain.partition_point(|&(other, _)| self.effects.id(other) < id);
                self.core.quick_f_chain.insert(index, (effect, chain));
            }
        }
    }

    // The core keeps delayed quick effects in a set ordered by effect address then event; effect id and event id
    // stand in for the addresses, which follow allocation order.
    fn insert_delayed_quick(&mut self, effect: EffectId, event: Event) {
        let key = |duel: &Self, (effect, event): (EffectId, Event)| (duel.effects.id(effect), event.global_id);
        let new_key = key(self, (effect, event));
        let delayed = &self.core.delayed_quick_tmp;
        if delayed.contains(&(effect, event)) {
            return;
        }
        let index = delayed.partition_point(|&entry| key(self, entry) <= new_key);
        self.core.delayed_quick_tmp.insert(index, (effect, event));
    }

    // A copy of a field container's equal range, for loops whose bodies may change the containers.
    #[inline(always)]
    pub(crate) fn field_effect_range(&self, container: Container, code: u32) -> EffectSet {
        if !self.field_effects.registry.has_code(code) {
            return EffectSet::default();
        }
        self.field_effect_range_lookup(container, code)
    }

    fn field_effect_range_lookup(&self, container: Container, code: u32) -> EffectSet {
        self.field_effects.registry.equal_range(container, None, code).collect()
    }

    fn check_simultaneous(&self, effect: EffectId, handler: CardId) -> bool {
        self.effects[effect].is_flag2(EFFECT_FLAG2_CHECK_SIMULTANEOUS) && self.core.just_sent_cards.contains(handler)
    }

    // A trigger collected by process_instant_event, with the player the core gives it.
    fn new_trigger(&mut self, effect: EffectId, event: &Event, was_just_sent: bool) -> Chain {
        self.set_activate_location(effect);
        let handler = self.get_handler(effect);
        let mut chain = Chain::new(effect);
        chain.was_just_sent = was_just_sent;
        chain.chain_id = self.next_field_id() as u16;
        chain.event_id = event.global_id;
        chain.evt = *event;
        self.set_triggering_state(&mut chain, handler);
        chain.triggering_player = if self.effects[effect].is_flag(EFFECT_FLAG_EVENT_PLAYER) && event.event_player < 2 {
            event.event_player
        } else {
            self.cards[handler].current.controler
        };
        chain
    }

    // Continuous effects with EFFECT_FLAG_DELAY wait while a chain or another continuous effect solves.
    fn continuous_chain_list<'a>(
        &'a mut self,
        effect: EffectId,
        owner_player: u8,
        tp: &'a mut Vec<Chain>,
        ntp: &'a mut Vec<Chain>,
    ) -> &'a mut Vec<Chain> {
        let turn_player = owner_player == self.infos.turn_player;
        if self.effects[effect].is_flag(EFFECT_FLAG_DELAY) && (self.core.chain_solving || self.core.conti_solving) {
            if turn_player {
                &mut self.core.delayed_continuous_tp
            } else {
                &mut self.core.delayed_continuous_ntp
            }
        } else if turn_player {
            tp
        } else {
            ntp
        }
    }

    fn push_continuous(&mut self, mut tp: Vec<Chain>, mut ntp: Vec<Chain>) {
        for chain in tp.drain(..).chain(ntp.drain(..)) {
            self.core.sub_solving_continuous.push(chain);
            self.emplace_process(SolveContinuous::default());
        }
        (self.core.continuous_tp, self.core.continuous_ntp) = (tp, ntp);
    }

    pub(crate) fn process_single_event(&mut self) {
        if self.core.single_event.is_empty() {
            return;
        }
        let (mut tp, mut ntp) = (
            std::mem::take(&mut self.core.continuous_tp),
            std::mem::take(&mut self.core.continuous_ntp),
        );
        for index in 0..self.core.single_event.len() {
            let Some(event) = self.core.single_event.get(index).copied() else {
                break;
            };
            let Some(target) = event.trigger_card else { continue };
            let mut effects: EffectSet = self
                .field_effects
                .registry
                .equal_range(Container::Single, Some(target), event.event_code)
                .collect();
            effects.extend(self.xmaterial_effects(target, event.event_code));
            for effect in effects {
                self.process_single_event_effect(effect, &event, &mut tp, &mut ntp);
            }
        }
        self.push_continuous(tp, ntp);
        self.core.single_event.clear();
    }

    fn process_single_event_effect(&mut self, effect: EffectId, event: &Event, tp: &mut Vec<Chain>, ntp: &mut Vec<Chain>) {
        let effect_type = self.effects[effect].effect_type;
        if effect_type & EFFECT_TYPE_ACTIONS == 0
            || (effect_type & EFFECT_TYPE_FLIP != 0 && event.event_value & (NO_FLIP_EFFECT >> 16) != 0)
        {
            return;
        }
        if effect_type & EFFECT_TYPE_CONTINUOUS != 0 {
            let mut owner_player = self.get_handler_player(effect);
            if self.effects[effect].is_flag(EFFECT_FLAG_EVENT_PLAYER) && event.event_player < 2 {
                owner_player = event.event_player;
            }
            if self.is_activateable(effect, owner_player, event, false, false, false, false, false) {
                let chain = Chain {
                    triggering_effect: effect,
                    triggering_player: owner_player,
                    evt: *event,
                    target_player: PLAYER_NONE,
                    disable_player: PLAYER_NONE,
                    ..Chain::default()
                };
                self.continuous_chain_list(effect, owner_player, tp, ntp).push(chain);
            }
            return;
        }
        let handler = self.get_handler(effect);
        if !self.is_condition_check(effect, self.cards[handler].current.controler, event) {
            return;
        }
        self.set_activate_location(effect);
        let mut chain = Chain::new(effect);
        chain.chain_id = self.next_field_id() as u16;
        chain.event_id = event.global_id;
        chain.evt = *event;
        self.set_triggering_state(&mut chain, handler);
        chain.triggering_player = if self.effects[effect].is_flag(EFFECT_FLAG_EVENT_PLAYER) && event.event_player < 2 {
            event.event_player
        } else if self.cards[handler].current.reason & REASON_TEMPORARY != 0 {
            self.cards[handler].previous.controler
        } else {
            chain.triggering_controler
        };
        self.set_active_type(effect);
        self.create_relation(handler, effect, chain.chain_id);
        if let Some(deffect) = self.is_affected_by_effect(handler, EFFECT_DISABLE_EFFECT) {
            self.add_disable_chain(handler, deffect, chain.chain_id, 0);
        }
        let optional = effect_type & EFFECT_TYPE_TRIGGER_O != 0;
        let list = match (self.core.flip_delayed && event.event_code == EVENT_FLIP, optional) {
            (true, true) => &mut self.core.new_ochain_b,
            (true, false) => &mut self.core.new_fchain_b,
            (false, true) => &mut self.core.new_ochain,
            (false, false) => &mut self.core.new_fchain,
        };
        list.push(chain);
    }

    pub(crate) fn solve_continuous(&mut self, playerid: u8, effect: EffectId, event: Event) {
        self.core.sub_solving_continuous.push(Chain {
            triggering_effect: effect,
            triggering_player: playerid,
            evt: event,
            target_player: PLAYER_NONE,
            disable_player: PLAYER_NONE,
            ..Chain::default()
        });
        self.emplace_process(SolveContinuous::default());
    }

    // ExecuteCost, ExecuteTarget and ExecuteOperation: the script function runs whole here (see script.rs); while one
    // of its prompts waits, the unit stays at step 0 under it.
    pub(crate) fn process_execute(&mut self, arg: &mut Execute) -> bool {
        let (effect, player, kind) = (arg.triggering_effect, arg.triggering_player, arg.kind);
        let peffect = &self.effects[effect];
        let has_function = match kind {
            ExecuteKind::Cost => peffect.cost.is_some(),
            ExecuteKind::Target => matches!(peffect.target, Target::Action(_)),
            ExecuteKind::Operation => peffect.operation.is_some(),
        };
        if !has_function {
            self.splice_sub_solving_event();
            self.core.solving_event.remove(0);
            return true;
        }
        let procedure = arg.procedure.as_deref().copied();
        let promptless = kind == ExecuteKind::Operation && matches!(peffect.operation, Some(Operation::Promptless(_)));
        let result = self.run_script(effect, player, promptless, |duel| {
            duel.splice_sub_solving_event();
            let event = duel.core.solving_event.first().copied().unwrap_or_default();
            let args = match procedure {
                Some(Procedure {
                    card,
                    must,
                    materials,
                    min,
                    max,
                    chkf,
                    zone,
                    releasable,
                }) => Args {
                    c: Some(card),
                    must,
                    materials,
                    min,
                    max,
                    chkf,
                    zone,
                    releasable,
                    ..Args::event(effect, player, &event)
                },
                None => Args::event(effect, player, &event),
            };
            if duel.core.check_level == 0 {
                duel.core.shuffle_deck_check = Default::default();
                duel.core.shuffle_hand_check = Default::default();
            }
            let shuffle_check_was_disabled = duel.core.shuffle_check_disabled;
            duel.core.shuffle_check_disabled = false;
            duel.core.check_level += 1;
            duel.core.reason_effect.set(Some(effect));
            duel.core.reason_player.set(player);
            let peffect = &duel.effects[effect];
            let value = match kind {
                ExecuteKind::Cost => match peffect.cost {
                    Some(cost) => i32::from(cost(duel, &args, 1)?),
                    None => 0,
                },
                ExecuteKind::Target => match peffect.target {
                    Target::Action(target) => i32::from(target(duel, &args, 1, None)?),
                    _ => 0,
                },
                ExecuteKind::Operation => match peffect.operation {
                    Some(Operation::Event(operation) | Operation::Promptless(operation)) => {
                        operation(duel, &args)?;
                        0
                    }
                    Some(Operation::Returning(operation)) => operation(duel, &args)? as i32,
                    Some(Operation::Group(operation)) => {
                        let mut group = duel.core.cards_to_summon_g;
                        operation(duel, &args, &mut group)?;
                        duel.core.cards_to_summon_g = group;
                        0
                    }
                    Some(Operation::SynchroMaterial(_) | Operation::ExtraMaterial(_) | Operation::CodeCondition(_)) | None => 0,
                },
            };
            duel.returns.set_i32(0, value);
            duel.core.reason_effect.set(None);
            duel.core.reason_player.set(PLAYER_NONE);
            duel.core.check_level -= 1;
            if duel.core.check_level == 0 {
                for p in 0..2 {
                    if duel.core.shuffle_hand_check[p] {
                        duel.shuffle(p, LOCATION_HAND);
                    }
                }
                for p in 0..2 {
                    if duel.core.shuffle_deck_check[p] {
                        duel.shuffle(p, LOCATION_DECK);
                    }
                }
            }
            duel.core.shuffle_check_disabled = shuffle_check_was_disabled;
            duel.core.solving_event.remove(0);
            Ok(())
        });
        if result.is_some() || self.result.is_some() {
            return true;
        }
        // A prompt waits: the unit runs again once it is answered.
        arg.step = RESTART;
        if self.core.replay.stack.is_empty() {
            self.core.presented = self.core.replay.prompt.take();
        }
        false
    }

    fn splice_sub_solving_event(&mut self) {
        let core = &mut self.core;
        for (index, &event) in core.sub_solving_event.iter().enumerate() {
            core.solving_event.insert(index, event);
        }
        core.sub_solving_event.clear();
    }

    pub(crate) fn execute(&mut self, kind: ExecuteKind, effect: EffectId, player: u8, event: Event) {
        self.execute_procedure(kind, effect, player, event, None);
    }

    pub(crate) fn execute_procedure(
        &mut self,
        kind: ExecuteKind,
        effect: EffectId,
        player: u8,
        event: Event,
        procedure: Option<Procedure>,
    ) {
        self.core.sub_solving_event.push(event);
        self.emplace_process(Execute {
            step: 0,
            kind,
            triggering_effect: effect,
            triggering_player: player,
            procedure: procedure.map(Box::new),
        });
    }

    pub(crate) fn process_add_chain(&mut self, arg: &mut AddChain) -> bool {
        match arg.step {
            0 => {
                let Some(chain) = self.core.new_chains.first() else { return true };
                arg.is_activated_effect = false;
                if self.effects[chain.triggering_effect].effect_type & EFFECT_TYPE_ACTIVATE != 0 {
                    arg.step = 9;
                }
                false
            }
            1 => {
                let Some(&chain) = self.core.new_chains.first() else { return true };
                for cost in self.activate_cost_effects(chain.triggering_effect, chain.triggering_player) {
                    if self.effects[cost].operation.is_some() {
                        self.execute(ExecuteKind::Operation, cost, chain.triggering_player, chain.evt);
                    }
                }
                if !arg.is_activated_effect {
                    return false;
                }
                let effect = chain.triggering_effect;
                let handler = self.get_handler(effect);
                let current = self.cards[handler].current;
                self.cards[handler].set_status(STATUS_ACT_FROM_HAND, current.location == LOCATION_HAND);
                if current.location == LOCATION_SZONE {
                    self.field_change_position(handler, None, current.controler, POS_FACEUP, 0, false);
                } else {
                    let data_type = self.cards[handler].data.card_type;
                    unported!(
                        self,
                        (self.effects[effect].is_flag(EFFECT_FLAG_LIMIT_ZONE) && data_type & (TYPE_FIELD | TYPE_PENDULUM) == 0)
                            .then_some(effect)
                    );
                    unported!(
                        self,
                        (self.effects[effect].is_flag2(EFFECT_FLAG2_FORCE_ACTIVATE_LOCATION)).then_some(effect)
                    );
                    let mut location = u16::from(LOCATION_SZONE);
                    if current.location == LOCATION_HAND {
                        if data_type & TYPE_PENDULUM != 0 {
                            location = LOCATION_PZONE;
                        } else if data_type & TYPE_FIELD != 0 {
                            location = LOCATION_FZONE;
                        }
                    }
                    self.enable_field_effect(handler, false);
                    let positions = if location == u16::from(LOCATION_MZONE) {
                        POS_FACEUP_ATTACK
                    } else {
                        POS_FACEUP
                    } as u8;
                    self.field_move_to_field(
                        handler,
                        current.controler,
                        current.controler,
                        location,
                        positions,
                        false,
                        0xff,
                        false,
                        LOCATION_REASON_NONE,
                    );
                }
                false
            }
            2 => {
                let Some(mut chain) = self.core.new_chains.first().copied() else {
                    return true;
                };
                let effect = chain.triggering_effect;
                let handler = self.get_handler(effect);
                if self.effects[effect].effect_type & EFFECT_TYPE_ACTIVATE != 0 {
                    self.set_triggering_state(&mut chain, handler);
                }
                let (code, location, description) = (
                    self.cards[handler].data.code,
                    self.cards.get_info_location(handler),
                    self.effects[effect].description,
                );
                self.trace
                    .message(MSG_CHAINING)
                    .u32(code)
                    .loc(location)
                    .u8(chain.triggering_controler)
                    .u8(chain.triggering_location as u8)
                    .u32(chain.triggering_sequence)
                    .u64(description)
                    .u32(self.core.current_chain.len() as u32 + 1);
                self.core.chain_limit.clear();
                let mut card_type = self.get_type(handler);
                if card_type & (TYPE_TRAP | TYPE_MONSTER) == TYPE_TRAP | TYPE_MONSTER {
                    card_type -= TYPE_TRAP;
                }
                self.effects[effect].card_type = card_type;
                let peffect = &self.effects[effect];
                if !self.is_flag(DUEL_TRIGGER_WHEN_PRIVATE_KNOWLEDGE)
                    || !(peffect.effect_type & EFFECT_TYPE_CONTINUOUS == 0 && peffect.effect_type & EFFECT_TYPE_SINGLE != 0)
                {
                    self.set_active_type(effect);
                }
                let active_handler = self.cards[self.effects[effect].handler.unwrap_or_default()].overlay_target;
                self.effects[effect].active_handler = active_handler;
                chain.chain_count = self.core.current_chain.len() as u8 + 1;
                chain.target_cards = None;
                chain.target_player = PLAYER_NONE;
                chain.target_param = 0;
                chain.disable_reason = None;
                chain.disable_player = PLAYER_NONE;
                chain.replace_op = None;
                if self.cards[handler].current.location == LOCATION_HAND {
                    chain.flag |= CHAIN_HAND_EFFECT;
                }
                self.core.current_chain.push(chain);
                let applied = self.check_chain_counter(effect, chain.triggering_player, chain.chain_count);
                if let Some(link) = self.core.current_chain.last_mut() {
                    link.applied_chain_counters = applied;
                }
                let peffect = &self.effects[effect];
                if !peffect.is_flag(EFFECT_FLAG_FIELD_ONLY)
                    && (peffect.effect_type & 0x2a0 == 0 || peffect.code & 0xffff_f000 == EVENT_PHASE)
                {
                    self.create_relation(handler, effect, chain.chain_id);
                }
                self.effects[effect].effect_owner = chain.triggering_player;
                if !self.effects[effect].is_flag(EFFECT_FLAG_FIELD_ONLY)
                    && self.is_has_relation(handler, effect, chain.chain_id)
                    && let Some(deffect) = self.is_affected_by_effect(handler, EFFECT_DISABLE_EFFECT)
                {
                    self.add_disable_chain(handler, deffect, chain.chain_id, RESET_CHAIN);
                }
                self.core.new_chains.remove(0);
                false
            }
            3 => {
                let Some(&chain) = self.core.current_chain.last() else {
                    return true;
                };
                let (effect, player) = (chain.triggering_effect, chain.triggering_player);
                if self.is_flag(DUEL_USE_TRAPS_IN_NEW_CHAIN) || !self.get_cteffect(effect, player, true) {
                    self.returns.set_i32(0, 0);
                    return false;
                }
                let peffect = &self.effects[effect];
                let phase = self.infos.phase;
                if (phase == PHASE_DAMAGE && !peffect.is_flag(EFFECT_FLAG_DAMAGE_STEP))
                    || (phase == PHASE_DAMAGE_CAL && !peffect.is_flag(EFFECT_FLAG_DAMAGE_CAL))
                {
                    self.returns.set_i32(0, 1);
                    return false;
                }
                let handler = self.get_handler(effect);
                self.field_select_effect_yes_no(player, 94, handler);
                false
            }
            4 => {
                if self.returns.at_i32(0) == 0 {
                    self.core.select_chains.clear();
                    self.core.select_options.clear();
                    arg.step = 5;
                    return false;
                }
                match self.core.current_chain.last() {
                    Some(chain) if self.core.select_chains.len() > 1 => {
                        let playerid = chain.triggering_player;
                        self.emplace_process(SelectOption {
                            playerid,
                            ..SelectOption::default()
                        });
                    }
                    _ => self.returns.set_i32(0, 0),
                }
                false
            }
            // The activation of a continuous trap that also applies one of its effects (get_cteffect).
            5 => {
                let selected = usize::try_from(self.returns.at_i32(0)).unwrap_or(0);
                let (Some(chain), Some(&choice)) = (self.core.current_chain.last().copied(), self.core.select_chains.get(selected)) else {
                    return true;
                };
                let (playerid, effect) = (chain.triggering_player, choice.triggering_effect);
                // The core picks the option to announce with an inverted bounds check, so every valid choice shows 65.
                self.trace.message(MSG_HINT).u8(HINT_OPSELECTED).u8(playerid).u64(65);
                let handler = self.get_handler(effect);
                if let Some(link) = self.core.current_chain.last_mut() {
                    link.triggering_effect = effect;
                    link.evt = choice.evt;
                }
                self.create_relation(handler, effect, chain.chain_id);
                self.dec_count(effect, playerid);
                if self.effects[effect].effect_type & EFFECT_TYPE_ACTIVATE == 0 {
                    self.effects[effect].effect_type |= EFFECT_TYPE_ACTIVATE;
                    if let Some(link) = self.core.current_chain.last_mut() {
                        link.flag |= CHAIN_ACTIVATING;
                    }
                }
                self.core.select_chains.clear();
                self.core.select_options.clear();
                self.add_hint_effect(handler, 65, RESET_CHAIN);
                false
            }
            6 => {
                let Some(&chain) = self.core.current_chain.last() else {
                    return true;
                };
                if self.effects[chain.triggering_effect].cost.is_some() {
                    self.execute(ExecuteKind::Cost, chain.triggering_effect, chain.triggering_player, chain.evt);
                }
                false
            }
            7 => {
                let Some(&chain) = self.core.current_chain.last() else {
                    return true;
                };
                if matches!(self.effects[chain.triggering_effect].target, Target::Action(_)) {
                    self.execute(ExecuteKind::Target, chain.triggering_effect, chain.triggering_player, chain.evt);
                }
                false
            }
            8 => {
                self.field_break_effect(false);
                let Some(&chain) = self.core.current_chain.last() else {
                    return true;
                };
                let effect = chain.triggering_effect;
                let handler = self.get_handler(effect);
                if let Some(targets) = chain.target_cards.filter(|targets| !targets.is_empty())
                    && self.effects[effect].is_flag(EFFECT_FLAG_CARD_TARGET)
                {
                    for card in targets {
                        self.field_raise_single_event(
                            card,
                            EVENT_BECOME_TARGET,
                            Some(effect),
                            0,
                            chain.triggering_player,
                            0,
                            u32::from(chain.chain_count),
                        );
                    }
                    self.process_single_event();
                    if let Some(targets) = self
                        .core
                        .current_chain
                        .last()
                        .and_then(|chain| chain.target_cards)
                        .filter(|t| !t.is_empty())
                    {
                        self.field_raise_event(
                            Some(targets),
                            EVENT_BECOME_TARGET,
                            Some(effect),
                            0,
                            chain.triggering_player,
                            chain.triggering_player,
                            u32::from(chain.chain_count),
                        );
                    }
                }
                let data_type = self.cards[handler].data.card_type;
                if self.effects[effect].effect_type & EFFECT_TYPE_ACTIVATE != 0 {
                    self.core.leave_confirmed.insert(handler);
                    if data_type & (TYPE_CONTINUOUS | TYPE_FIELD | TYPE_EQUIP | TYPE_PENDULUM | TYPE_LINK) == 0
                        && self.is_affected_by_effect(handler, EFFECT_REMAIN_FIELD).is_none()
                    {
                        self.cards[handler].set_status(STATUS_LEAVE_CONFIRMED, true);
                    }
                }
                let handler_type = self.get_type(handler);
                if handler_type & (TYPE_SPELL | TYPE_TRAP) != 0
                    && handler_type & (TYPE_CONTINUOUS | TYPE_FIELD | TYPE_EQUIP | TYPE_PENDULUM | TYPE_LINK) != 0
                    && self.is_has_relation(handler, effect, chain.chain_id)
                    && self.cards[handler].current.location == LOCATION_SZONE
                    && !self.effects[effect].is_flag(EFFECT_FLAG_FIELD_ONLY)
                    && let Some(last) = self.core.current_chain.last_mut()
                {
                    last.flag |= CHAIN_CONTINUOUS_CARD;
                }
                self.core.phase_action = true;
                if let Some(last) = self.core.current_chain.last() {
                    unported!(
                        self,
                        ({
                            last.opinfos.get(CATEGORY_SPECIAL_SUMMON).is_some_and(|info| info.op_count != 0)
                                && self.is_flag(DUEL_CANNOT_SUMMON_OATH_OLD)
                        })
                        .then_some(last.triggering_effect)
                    );
                }
                self.trace.message(MSG_CHAINED).u8(chain.chain_count);
                self.field_raise_event(
                    Some(CardSet::from(handler)),
                    EVENT_CHAINING,
                    Some(effect),
                    0,
                    chain.triggering_player,
                    chain.triggering_player,
                    u32::from(chain.chain_count),
                );
                self.process_instant_event();
                self.core.just_sent_cards.clear();
                self.core.real_chain_count += 1;
                if !self.core.new_chains.is_empty() {
                    self.emplace_process(AddChain::default());
                }
                self.adjust_all();
                true
            }
            10 => {
                arg.is_activated_effect = true;
                let Some(&chain) = self.core.new_chains.first() else { return true };
                let handler = self.get_handler(chain.triggering_effect);
                let card = self.cards[handler];
                let data_type = card.data.card_type;
                let quick = data_type & TYPE_QUICKPLAY != 0 || self.is_affected_by_effect(handler, EFFECT_BECOME_QUICK).is_some();
                let ecode = if card.current.location == LOCATION_HAND {
                    if data_type & TYPE_TRAP != 0 {
                        EFFECT_TRAP_ACT_IN_HAND
                    } else if data_type & TYPE_SPELL != 0 && quick && self.infos.turn_player != card.current.controler {
                        EFFECT_QP_ACT_IN_NTPHAND
                    } else {
                        0
                    }
                } else if card.current.location == LOCATION_SZONE {
                    let mut ecode = 0;
                    if data_type & TYPE_TRAP != 0 && card.get_status(STATUS_SET_TURN) {
                        ecode = EFFECT_TRAP_ACT_IN_SET_TURN;
                    }
                    if data_type & TYPE_SPELL != 0 && quick && card.get_status(STATUS_SET_TURN) {
                        ecode = EFFECT_QP_ACT_IN_SET_TURN;
                    }
                    ecode
                } else {
                    0
                };
                if ecode == 0 {
                    arg.step = 0;
                    return false;
                }
                let controler = card.current.controler;
                self.core.select_effects.clear();
                self.core.select_options.clear();
                let effects = self.filter_effect(handler, ecode, true);
                if effects.is_empty() {
                    return false;
                }
                let mut has_no_side_effect_effs = false;
                for effect in effects {
                    let peffect = &self.effects[effect];
                    if !(peffect.is_flag(EFFECT_FLAG_FUNC_VALUE) || peffect.is_flag(EFFECT_FLAG_COUNT_LIMIT)) {
                        has_no_side_effect_effs = true;
                    } else if self.check_count_limit(effect, controler) {
                        let description = peffect.description;
                        self.core.select_effects.push(effect);
                        self.core.select_options.push(description);
                    }
                }
                if self.core.select_options.is_empty() {
                    arg.step = 0;
                    return false;
                }
                // The option 99 that stands for the effects with no side effect has no entry in select_effects.
                if has_no_side_effect_effs {
                    self.core.select_options.insert(0, 99);
                }
                if self.core.select_options.len() == 1 {
                    self.returns.set_i32(0, 0);
                } else {
                    self.emplace_process(SelectOption {
                        playerid: controler,
                        ..SelectOption::default()
                    });
                }
                false
            }
            11 => {
                let Some(&chain) = self.core.new_chains.first() else { return true };
                let handler = self.get_handler(chain.triggering_effect);
                let offset = self.core.select_options.len().saturating_sub(self.core.select_effects.len());
                let selected = usize::try_from(self.returns.at_i32(0))
                    .ok()
                    .and_then(|index| index.checked_sub(offset));
                if let Some(&effect) = selected.and_then(|index| self.core.select_effects.get(index)) {
                    self.dec_count(effect, self.cards[handler].current.controler);
                    match self.effects[effect].value {
                        Value::Activation(value) => value(self, effect, chain.triggering_effect),
                        _ => debug_assert!(false, "an activation condition read with the wrong parameters"),
                    }
                }
                arg.step = 0;
                false
            }
            _ => true,
        }
    }

    pub(crate) fn process_solve_continuous(&mut self, arg: &mut SolveContinuous) -> bool {
        match arg.step {
            0 => {
                let core = &mut self.core;
                core.solving_continuous.splice(0..0, core.sub_solving_continuous.drain(..));
                let Some(&chain) = self.core.solving_continuous.first() else {
                    return true;
                };
                let (effect, player) = (chain.triggering_effect, chain.triggering_player);
                if !self.check_count_limit(effect, player) {
                    self.core.solving_continuous.remove(0);
                    return true;
                }
                self.core.continuous_chain.push(chain);
                if self.delays_continuous(effect) {
                    self.core.conti_solving = true;
                }
                arg.reason_effect = self.core.reason_effect.get();
                arg.reason_player = self.core.reason_player.get();
                if matches!(self.effects[effect].target, Target::Action(_)) {
                    self.execute(ExecuteKind::Target, effect, player, chain.evt);
                }
                false
            }
            1 => false,
            2 => {
                let Some(&chain) = self.core.solving_continuous.first() else {
                    return true;
                };
                let (effect, player) = (chain.triggering_effect, chain.triggering_player);
                if self.effects[effect].operation.is_none() {
                    return false;
                }
                self.dec_count(effect, player);
                self.execute(ExecuteKind::Operation, effect, player, chain.evt);
                false
            }
            3 => {
                let Some(&chain) = self.core.solving_continuous.first() else {
                    return true;
                };
                self.core.reason_effect.set(arg.reason_effect);
                self.core.reason_player.set(arg.reason_player);
                self.core.continuous_chain.pop();
                self.core.solving_continuous.remove(0);
                if self.delays_continuous(chain.triggering_effect) {
                    self.core.conti_solving = false;
                    self.adjust_all();
                    return false;
                }
                true
            }
            4 => {
                let turn_player = self.infos.turn_player;
                if self.core.conti_player == PLAYER_NONE {
                    self.core.conti_player = turn_player;
                }
                if self.core.conti_player == turn_player {
                    if !self.core.delayed_continuous_tp.is_empty() {
                        let chain = self.core.delayed_continuous_tp.remove(0);
                        self.core.sub_solving_continuous.push(chain);
                        self.emplace_process(SolveContinuous::default());
                    } else {
                        self.core.conti_player = 1 - turn_player;
                    }
                }
                if self.core.conti_player == 1 - turn_player {
                    if !self.core.delayed_continuous_ntp.is_empty() {
                        let chain = self.core.delayed_continuous_ntp.remove(0);
                        self.core.sub_solving_continuous.push(chain);
                        self.emplace_process(SolveContinuous::default());
                    } else if !self.core.delayed_continuous_tp.is_empty() {
                        self.core.conti_player = turn_player;
                        let chain = self.core.delayed_continuous_tp.remove(0);
                        self.core.sub_solving_continuous.push(chain);
                        self.emplace_process(SolveContinuous::default());
                    } else {
                        self.core.conti_player = PLAYER_NONE;
                    }
                }
                true
            }
            _ => true,
        }
    }

    fn delays_continuous(&self, effect: EffectId) -> bool {
        let peffect = &self.effects[effect];
        peffect.is_flag(EFFECT_FLAG_DELAY) || (peffect.code & 0xffff_f000 == 0 && peffect.code & (EVENT_PHASE | EVENT_PHASE_START) != 0)
    }

    pub(crate) fn process_solve_chain(&mut self, arg: &mut SolveChain) -> bool {
        if self.core.current_chain.is_empty() && arg.step == 0 {
            return true;
        }
        // Steps 11 to 13 run once the chain is empty.
        let chain = self.core.current_chain.last().copied().unwrap_or_default();
        let (effect, player, count) = (chain.triggering_effect, chain.triggering_player, chain.chain_count);
        match arg.step {
            0 => {
                unported!(self, (self.core.spsummon_rst).then_some(effect));
                self.trace.message(MSG_CHAIN_SOLVING).u8(count);
                self.add_to_disable_check_list(self.get_handler(effect));
                self.adjust_instant();
                self.field_raise_event(None, EVENT_CHAIN_ACTIVATING, Some(effect), 0, player, player, u32::from(count));
                self.process_instant_event();
                false
            }
            1 => {
                if chain.flag & CHAIN_DISABLE_ACTIVATE != 0 && self.field_is_chain_negatable(count) {
                    self.remove_oath_effect(effect);
                    self.restore_chain_counter(player, chain.applied_chain_counters);
                    let peffect = &self.effects[effect];
                    if peffect.is_flag(EFFECT_FLAG_COUNT_LIMIT) && peffect.count_flag & EFFECT_COUNT_CODE_OATH != 0 {
                        let (code, flag, hopt) = (peffect.count_code, peffect.count_flag, peffect.count_hopt_index);
                        self.dec_effect_code(code, flag, hopt, player);
                    }
                    let negated = |other: &Chain| other.evt.event_code == EVENT_CHAINING && other.evt.event_value == u32::from(count);
                    self.core.new_fchain.retain(|other| !negated(other));
                    self.core.new_ochain.retain(|other| !negated(other));
                    self.field_raise_event(None, EVENT_CHAIN_NEGATED, Some(effect), 0, player, player, u32::from(count));
                    self.process_instant_event();
                    arg.step = 9;
                    return false;
                }
                self.release_oath_relation(effect);
                self.field_break_effect(true);
                self.core.chain_solving = true;
                self.field_raise_event(None, EVENT_CHAIN_SOLVING, Some(effect), 0, player, player, u32::from(count));
                self.process_instant_event();
                false
            }
            2 => {
                self.core.spsummon_state_count_tmp = self.core.spsummon_state_count;
                let card = self.get_handler(effect);
                let has_relation = self.is_has_relation(card, effect, chain.chain_id);
                if chain.flag & CHAIN_CONTINUOUS_CARD != 0 && !has_relation {
                    arg.step = 3;
                    return false;
                }
                if self.effects[effect].effect_type & EFFECT_TYPE_ACTIVATE != 0 && has_relation && chain.replace_op.is_none() {
                    self.enable_field_effect(card, true);
                    self.adjust_instant();
                }
                let continuous_target = self.effects[effect].is_flag(EFFECT_FLAG_CONTINUOUS_TARGET);
                if self.field_is_chain_disablable(count)
                    && (!continuous_target || chain.replace_op.is_some())
                    && (self.is_chain_disabled(count) || (self.cards[card].get_status(STATUS_DISABLED | STATUS_FORBIDDEN) && has_relation))
                {
                    if chain.flag & CHAIN_DISABLE_EFFECT == 0 {
                        self.trace.message(MSG_CHAIN_DISABLED).u8(count);
                    }
                    self.field_raise_event(None, EVENT_CHAIN_DISABLED, Some(effect), 0, player, player, u32::from(count));
                    self.process_instant_event();
                    arg.step = 3;
                }
                false
            }
            3 => {
                let card = self.get_handler(effect);
                if chain.flag & CHAIN_CONTINUOUS_CARD != 0 && !self.is_has_relation(card, effect, chain.chain_id) {
                    return false;
                }
                arg.backed_up_operation = None;
                if let Some(replace_op) = chain.replace_op {
                    arg.backed_up_operation = self.effects[effect].operation;
                    self.effects[effect].operation = Some(Operation::Event(replace_op));
                }
                if self.effects[effect].operation.is_some() {
                    self.execute(ExecuteKind::Operation, effect, player, chain.evt);
                }
                false
            }
            4 => {
                if let Some(operation) = arg.backed_up_operation {
                    self.effects[effect].operation = Some(operation);
                }
                self.core.special_summoning.clear();
                self.core.equiping_cards.clear();
                false
            }
            5 => {
                if arg.backed_up_operation.take().is_none() {
                    unported!(
                        self,
                        (chain.opinfos.get(CATEGORY_SPECIAL_SUMMON).is_some_and(|info| info.op_count != 0)
                            && self.is_flag(DUEL_CANNOT_SUMMON_OATH_OLD))
                        .then_some(effect)
                    );
                }
                self.core.spsummon_state_count_tmp = Default::default();
                self.core.chain_solving = false;
                let turn_player = self.infos.turn_player;
                if !self.core.delayed_continuous_tp.is_empty() {
                    self.core.conti_player = turn_player;
                    let delayed = self.core.delayed_continuous_tp.remove(0);
                    self.core.sub_solving_continuous.push(delayed);
                    self.emplace_process(SolveContinuous::default());
                } else if !self.core.delayed_continuous_ntp.is_empty() {
                    self.core.conti_player = 1 - turn_player;
                    let delayed = self.core.delayed_continuous_ntp.remove(0);
                    self.core.sub_solving_continuous.push(delayed);
                    self.emplace_process(SolveContinuous::default());
                } else {
                    self.core.conti_player = PLAYER_NONE;
                }
                self.field_raise_event(None, EVENT_CHAIN_SOLVED, Some(effect), 0, player, player, u32::from(count));
                self.adjust_disable_check_list();
                self.process_instant_event();
                arg.step = 9;
                false
            }
            10 => {
                self.trace.message(MSG_CHAIN_SOLVED).u8(count);
                let card = self.get_handler(effect);
                if self.effects[effect].effect_type & EFFECT_TYPE_ACTIVATE != 0 && chain.flag & CHAIN_ACTIVATING != 0 {
                    self.effects[effect].effect_type &= !EFFECT_TYPE_ACTIVATE;
                }
                let current = self.cards[card].current;
                if chain.flag & CHAIN_HAND_EFFECT != 0 && !self.cards[card].is_position(POS_FACEUP) && current.location == LOCATION_HAND {
                    self.shuffle(current.controler, LOCATION_HAND);
                }
                for target in chain.target_cards.unwrap_or_default() {
                    self.release_relation(target, effect, chain.chain_id);
                }
                let has_relation = self.is_has_relation(card, effect, chain.chain_id);
                if self.cards[card].data.card_type & TYPE_EQUIP != 0
                    && self.effects[effect].effect_type & EFFECT_TYPE_ACTIVATE != 0
                    && self.cards[card].equiping_target.is_none()
                    && has_relation
                {
                    self.cards[card].set_status(STATUS_LEAVE_CONFIRMED, true);
                }
                self.effects[effect].active_type = 0;
                self.effects[effect].active_handler = None;
                self.release_relation(card, effect, chain.chain_id);
                for delayed in std::mem::take(&mut self.core.delayed_enable_set) {
                    if self.cards[delayed].current.location == LOCATION_MZONE {
                        self.enable_field_effect(delayed, true);
                    }
                }
                self.adjust_all();
                self.core.current_chain.pop();
                self.core.real_chain_count = (self.core.real_chain_count - 1).max(0);
                if self.core.current_chain.is_empty() {
                    self.core.chain_limit.clear();
                    return false;
                }
                self.run_reserved(false);
                arg.step = RESTART;
                false
            }
            11 => {
                let mut leave = CardSet::default();
                for card in self.core.leave_confirmed {
                    if self.cards[card].is_status(STATUS_LEAVE_CONFIRMED) {
                        leave.insert(card);
                    }
                }
                self.core.leave_confirmed = leave;
                if !leave.is_empty() {
                    self.send_to(
                        leave,
                        None,
                        REASON_RULE,
                        PLAYER_NONE,
                        PLAYER_NONE,
                        LOCATION_GRAVE,
                        0,
                        POS_FACEUP as u8,
                        false,
                    );
                }
                false
            }
            12 => {
                let core = &mut self.core;
                core.point_event.clear();
                self.trace.message(MSG_CHAIN_END);
                self.reset_chain();
                self.run_reserved(true);
                false
            }
            13 => {
                self.core.just_sent_cards.clear();
                self.field_raise_event(None, EVENT_CHAIN_END, None, 0, 0, 0, 0);
                self.process_instant_event();
                self.adjust_all();
                if !arg.skip_trigger || !arg.skip_new {
                    self.core.hint_timing[0] |= TIMING_CHAIN_END;
                    self.core.hint_timing[1] |= TIMING_CHAIN_END;
                    self.emplace_process(PointEvent {
                        skip_trigger: arg.skip_trigger,
                        skip_freechain: arg.skip_freechain || arg.skip_new,
                        skip_new: arg.skip_new,
                        ..PointEvent::default()
                    });
                }
                self.returns.set_i32(0, 1);
                true
            }
            _ => true,
        }
    }

    // The procedure summon a resolved chain link started (Duel.SynchroSummon and friends).
    // `damage_step` also resumes the damage step of Duel.CalculateDamage, which waits for the chain to end.
    fn run_reserved(&mut self, damage_step: bool) {
        let summoning = self.core.summoning_card.take().is_some();
        if (summoning || (damage_step && self.core.effect_damage_step == 1))
            && let Some(reserved) = self.core.reserved.take()
        {
            self.core.subunits.push(*reserved);
        }
    }

    pub(crate) fn field_break_effect(&mut self, clear_sent: bool) {
        if clear_sent {
            self.core.just_sent_cards.clear();
        }
        self.core.hint_timing[0] &= TIMING_DAMAGE_STEP | TIMING_DAMAGE_CAL;
        self.core.hint_timing[1] &= TIMING_DAMAGE_STEP | TIMING_DAMAGE_CAL;
        let ochain = std::mem::take(&mut self.core.new_ochain);
        for chain in ochain {
            let effect = chain.triggering_effect;
            let peffect = &self.effects[effect];
            if peffect.is_flag(EFFECT_FLAG_DELAY) {
                self.core.new_ochain.push(chain);
                continue;
            }
            if peffect.is_flag(EFFECT_FLAG_FIELD_ONLY)
                || peffect.effect_type & EFFECT_TYPE_FIELD == 0
                || self.in_range_chain(effect, &chain)
            {
                let handler = self.get_handler(effect);
                let (location, code) = (self.cards.get_info_location(handler), self.cards[handler].data.code);
                self.trace.message(MSG_MISSED_EFFECT).loc(location).u32(code);
            }
        }
        self.core.instant_event.clear();
        self.adjust_instant();
        if !self.core.force_turn_end {
            self.check_win();
        }
    }

    // The trigger and quick effect windows of PointEvent steps 2 and 4: the forced or optional triggers of the
    // current player that still apply, dropping the others from `list`.
    pub(crate) fn collect_triggers(&mut self, forced: bool) {
        self.core.select_chains.clear();
        let mut list = std::mem::take(if forced {
            &mut self.core.new_fchain_s
        } else {
            &mut self.core.new_ochain_s
        });
        let mut kept = 0;
        for index in 0..list.len() {
            let Some(mut chain) = list.get(index).copied() else { break };
            let effect = chain.triggering_effect;
            let handler = self.get_handler(effect);
            let mut updated = false;
            if self.is_has_relation(handler, effect, chain.chain_id) {
                self.set_triggering_state(&mut chain, handler);
                updated = true;
            }
            let peffect = &self.effects[effect];
            if !forced
                && !peffect.is_flag(EFFECT_FLAG_FIELD_ONLY)
                && peffect.effect_type & EFFECT_TYPE_FIELD != 0
                && peffect.range & u16::from(LOCATION_HAND) != 0
                && self.cards[handler].current.location == LOCATION_HAND
            {
                let controler = self.cards[handler].current.controler;
                if !self.is_has_relation(handler, effect, chain.chain_id) && self.is_condition_check(effect, controler, &chain.evt) {
                    self.create_relation(handler, effect, chain.chain_id);
                }
                self.set_activate_location(effect);
                chain.triggering_player = controler;
                if !updated {
                    self.set_triggering_state(&mut chain, handler);
                    updated = true;
                }
            }
            let controler = self.cards[handler].current.controler;
            if chain.triggering_player != controler && !self.effects[effect].is_flag(EFFECT_FLAG_EVENT_PLAYER) {
                chain.triggering_player = controler;
                if !updated {
                    self.set_triggering_state(&mut chain, handler);
                }
            }
            let tp = chain.triggering_player;
            let private = self.is_flag(DUEL_TRIGGER_WHEN_PRIVATE_KNOWLEDGE);
            let keep = (!chain.was_just_sent || self.cards[handler].current.location == LOCATION_HAND)
                && (forced || private || self.check_nonpublic_trigger(&mut chain))
                && self.check_trigger_effect(&chain)
                && self.is_chainable(effect, tp)
                && self.is_activateable(effect, tp, &chain.evt, true, false, false, private, private)
                && (forced || self.check_spself_from_hand_trigger(&chain));
            if keep {
                if tp == self.core.current_player {
                    self.core.select_chains.push(chain);
                }
                if let Some(slot) = list.get_mut(kept) {
                    *slot = chain;
                }
                kept += 1;
            } else {
                self.effects[effect].active_type = 0;
            }
        }
        list.truncate(kept);
        if forced {
            self.core.new_fchain_s = list;
        } else {
            self.core.new_ochain_s = list;
        }
    }

    // The TCG SEGOC rule: only the triggers of the earliest event can be chained first.
    pub(crate) fn segoc_first_trigger(&mut self) {
        if !self.is_flag(DUEL_TCG_SEGOC_FIRSTTRIGGER) {
            return;
        }
        let chains = &mut self.core.select_chains;
        chains.sort_by_key(|chain| chain.event_id);
        if let Some(first) = chains.first().map(|chain| chain.event_id) {
            chains.retain(|chain| chain.event_id == first);
        }
    }

    // Moves the chosen candidate to new_chains, as the core splices it.
    pub(crate) fn take_select_chain(&mut self, index: usize) -> Option<Chain> {
        (index < self.core.select_chains.len()).then(|| self.core.select_chains.remove(index))
    }

    // field::check_chain_counter: the ACTIVITY_CHAIN counters whose filter the new chain link fails count it.
    fn check_chain_counter(&mut self, effect: EffectId, playerid: u8, chain_count: u8) -> u8 {
        let mut applied = 0;
        for index in 0..self.core.custom_counters.len() {
            let Some(counter) = self.core.custom_counters.get(index).copied() else {
                break;
            };
            let CounterFilter::Chain(filter) = counter.filter else { continue };
            if counter.activity != ACTIVITY_CHAIN || filter(self, effect, playerid, chain_count) {
                continue;
            }
            let counter = self.core.custom_counters.get_mut(index);
            if let Some(amount) = counter.and_then(|counter| counter.amount.get_mut(usize::from(playerid))) {
                *amount += 1;
            }
            applied |= 1 << index;
        }
        applied
    }

    fn restore_chain_counter(&mut self, playerid: u8, applied: u8) {
        for (index, counter) in self.core.custom_counters.iter_mut().enumerate() {
            if applied & (1 << index) != 0
                && let Some(amount) = counter.amount.get_mut(usize::from(playerid))
            {
                *amount = amount.saturating_sub(1);
            }
        }
    }

    pub(crate) fn field_select_effect_yes_no(&mut self, playerid: u8, description: u64, card: CardId) {
        self.emplace_process(SelectEffectYesNo {
            playerid,
            description,
            card,
            ..SelectEffectYesNo::default()
        });
    }
}

impl Duel {
    // The EFFECT_DISABLE_CHAIN an EFFECT_DISABLE_EFFECT puts on the handler of a new chain link.
    fn add_disable_chain(&mut self, handler: CardId, deffect: EffectId, chain_id: u16, reset: u32) {
        let negate = self.new_effect(self.effects[deffect].owner);
        let value = self.get_value(deffect, handler) as u32;
        let peffect = &mut self.effects[negate];
        peffect.effect_type = EFFECT_TYPE_SINGLE;
        peffect.code = EFFECT_DISABLE_CHAIN;
        peffect.value = crate::effect::Value::Int(i32::from(chain_id));
        peffect.reset_flag = reset | RESET_EVENT | value;
        self.card_add_effect(handler, negate);
    }
}
