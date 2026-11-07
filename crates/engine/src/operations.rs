// Ports of operations.cpp.
use crate::card::{CardId, CardSet, SendtoParam};
use crate::chain::ExecuteKind;
use crate::constants::*;
use crate::duel::Duel;
use crate::effect::{Container, EffectId, EffectSet, Target, Value};
use crate::field::{Cards16, PerPlayer};
use crate::processor::{PointEvent, SelectCard, SelectOption, SelectPlace, SelectPosition, SelectTributeP, SelectUnselectCard};
use crate::script::Procedure;

#[derive(Debug, Clone, Default)]
pub struct Draw {
    pub step: u16,
    pub count: u16,
    pub reason_player: u8,
    pub playerid: u8,
    pub reason: u32,
    pub reason_effect: Option<EffectId>,
    pub drawn_set: CardSet,
}

// `remaining` stands for the core's iterator over the targets.
#[derive(Debug, Clone, Default)]
pub struct GetControl {
    pub step: u16,
    pub reason_effect: Option<EffectId>,
    pub chose_player: u8,
    pub targets: CardSet,
    pub playerid: u8,
    pub reset_phase: u16,
    pub reset_count: u8,
    pub zone: u32,
    pub destroy_set: CardSet,
    pub remaining: CardSet,
}

#[derive(Debug, Clone)]
pub struct Equip {
    pub step: u16,
    pub equip_player: u8,
    pub equip_card: CardId,
    pub target: CardId,
    pub faceup: bool,
    pub is_step: bool,
}

#[derive(Debug, Clone, Default)]
pub struct DiscardDeck {
    pub step: u16,
    pub playerid: u8,
    pub count: u16,
    pub reason: u32,
    pub discarded_set: CardSet,
}

// The core shares `targets` as a group with SendToReplace; here the replace checks run inline in step 1.
#[derive(Debug, Clone, Default)]
pub struct SendTo {
    pub step: u16,
    pub reason_player: u8,
    pub reason: u32,
    pub targets: CardSet,
    pub reason_effect: Option<EffectId>,
    pub extra_args: Option<Box<SendToArgs>>,
}

#[derive(Debug, Clone, Default)]
pub struct SendToArgs {
    pub leave_field: CardSet,
    pub leave_grave: CardSet,
    pub check_decktop_visibility: PerPlayer<bool>,
    pub cv: Cards16,
    pub cvit: usize,
    pub detach: CardSet,
}

// Battle enters at steps 3 and 10.
#[derive(Debug, Clone)]
pub struct Destroy {
    pub step: u16,
    pub reason_player: u8,
    pub reason: u32,
    pub targets: CardSet,
    pub reason_effect: Option<EffectId>,
}

// `is_step` and `is_reflected` come with EFFECT_REFLECT_DAMAGE and the effects that damage in steps.
#[derive(Debug, Clone)]
pub struct Damage {
    pub step: u16,
    pub reason_player: u8,
    pub playerid: u8,
    pub amount: u32,
    pub reason: u32,
    pub reason_card: Option<CardId>,
    pub reason_effect: Option<EffectId>,
}

// `is_step` comes with the effects that recover in steps.
#[derive(Debug, Clone)]
pub struct Recover {
    pub step: u16,
    pub reason_player: u8,
    pub playerid: u8,
    pub amount: u32,
    pub reason: u32,
    pub reason_effect: Option<EffectId>,
}

#[derive(Debug, Clone)]
pub struct SpellSet {
    pub step: u16,
    pub setplayer: u8,
    pub toplayer: u8,
    pub target: CardId,
    pub reason_effect: Option<EffectId>,
}

// The core shares `targets` as a group with the SpSummonStep units, which remove the cards they fail to summon;
// here step 1 drops the targets without STATUS_SPSUMMON_STEP, which only a step that succeeded sets.
#[derive(Debug, Clone)]
pub struct SpSummon {
    pub step: u16,
    pub reason_player: u8,
    pub zone: u32,
    pub reason_effect: Option<EffectId>,
    pub targets: CardSet,
}

// `in_group`: part of a SpSummon, rather than Duel.SpecialSummonStep's lone step.
#[derive(Debug, Clone)]
pub struct SpSummonStep {
    pub step: u16,
    pub zone: u32,
    pub in_group: bool,
    pub target: CardId,
}

#[derive(Debug, Clone)]
pub struct Release {
    pub step: u16,
    pub reason_player: u8,
    pub reason: u32,
    pub targets: CardSet,
    pub reason_effect: Option<EffectId>,
}

// `ret` 1 returns a temporarily banished card; 2, for trap monsters, comes with the cards that use it.
#[derive(Debug, Clone)]
pub struct MoveToField {
    pub step: u16,
    pub enable: bool,
    pub pzone: bool,
    pub zone: u8,
    pub rule: bool,
    pub location_reason: u32,
    pub confirm: bool,
    pub ret: u8,
    pub target: CardId,
}

#[derive(Debug, Clone)]
pub struct ChangePos {
    pub step: u16,
    pub reason_player: u8,
    pub enable: bool,
    pub oppo_selection: bool,
    pub reason_effect: Option<EffectId>,
    pub targets: CardSet,
    pub to_grave_set: CardSet,
}

#[derive(Debug, Clone)]
pub struct SummonRule {
    pub step: u16,
    pub sumplayer: u8,
    pub min_tribute: u8,
    pub ignore_count: bool,
    pub zone: u32,
    pub target: CardId,
    pub summon_procedure_effect: Option<EffectId>,
    pub extra_summon_effect: Option<EffectId>,
    pub tributes: CardSet,
    // Whether the ordinary summon comes first among the options, before core.select_effects.
    pub ordinary: bool,
}

#[derive(Debug, Clone)]
pub struct MonsterSet {
    pub step: u16,
    pub setplayer: u8,
    pub min_tribute: u8,
    pub ignore_count: bool,
    pub zone: u32,
    pub target: CardId,
    pub summon_procedure_effect: Option<EffectId>,
    pub extra_summon_effect: Option<EffectId>,
    pub tributes: CardSet,
    pub ordinary: bool,
}

#[derive(Debug, Clone)]
pub struct FlipSummon {
    pub step: u16,
    pub sumplayer: u8,
    pub target: CardId,
}

#[derive(Debug, Clone)]
pub struct SelectTribute {
    pub step: u16,
    pub min: u16,
    pub max: u16,
    pub playerid: u8,
    pub cancelable: bool,
    pub toplayer: u8,
    pub zone: u8,
    pub target: Option<CardId>,
    pub extra_release_effect: Option<EffectId>,
    pub must_choose_one: CardSet,
}

impl Duel {
    pub(crate) fn negate_chain(&mut self, chaincount: u8) -> bool {
        let len = self.core.current_chain.len();
        if len == 0 {
            return false;
        }
        let index = if chaincount == 0 || usize::from(chaincount) > len {
            len - 1
        } else {
            usize::from(chaincount) - 1
        };
        let Some(&chain) = self.core.current_chain.get(index) else {
            return false;
        };
        let handler = self.get_handler(chain.triggering_effect);
        if chain.flag & CHAIN_DISABLE_ACTIVATE != 0
            || !self.field_is_chain_negatable(chain.chain_count)
            || !self.is_affect_by_effect(handler, self.core.reason_effect.get())
        {
            return false;
        }
        let (reason_effect, reason_player) = (self.core.reason_effect.get(), self.core.reason_player.get());
        if let Some(link) = self.core.current_chain.get_mut(index) {
            link.flag |= CHAIN_DISABLE_ACTIVATE;
            link.disable_reason = reason_effect;
            link.disable_player = reason_player;
        }
        if self.effects[chain.triggering_effect].effect_type & EFFECT_TYPE_ACTIVATE != 0
            && self.is_has_relation(handler, chain.triggering_effect, chain.chain_id)
            && self.cards[handler].current.location == LOCATION_SZONE
        {
            self.cards[handler].set_status(STATUS_LEAVE_CONFIRMED, true);
            self.cards[handler].set_status(STATUS_ACTIVATE_DISABLED, true);
        }
        self.trace.message(MSG_CHAIN_NEGATED).u8(index as u8 + 1);
        if !self.is_flag(DUEL_RETURN_TO_DECK_TRIGGERS)
            && (chain.triggering_location == u16::from(LOCATION_DECK)
                || (chain.triggering_location == u16::from(LOCATION_EXTRA) && u32::from(chain.triggering_position) & POS_FACEDOWN != 0))
        {
            self.release_relation(handler, chain.triggering_effect, chain.chain_id);
        }
        true
    }

    pub(crate) fn disable_chain(&mut self, chaincount: u8) -> bool {
        let len = self.core.current_chain.len();
        if len == 0 {
            return false;
        }
        let index = if chaincount == 0 || usize::from(chaincount) > len {
            len - 1
        } else {
            usize::from(chaincount) - 1
        };
        let Some(&chain) = self.core.current_chain.get(index) else {
            return false;
        };
        let handler = self.get_handler(chain.triggering_effect);
        if chain.flag & CHAIN_DISABLE_EFFECT != 0
            || !self.field_is_chain_disablable(chain.chain_count)
            || !self.is_affect_by_effect(handler, self.core.reason_effect.get())
        {
            return false;
        }
        let (reason_effect, reason_player) = (self.core.reason_effect.get(), self.core.reason_player.get());
        if let Some(link) = self.core.current_chain.get_mut(index) {
            link.flag |= CHAIN_DISABLE_EFFECT;
            link.disable_reason = reason_effect;
            link.disable_player = reason_player;
        }
        self.trace.message(MSG_CHAIN_DISABLED).u8(index as u8 + 1);
        if !self.is_flag(DUEL_RETURN_TO_DECK_TRIGGERS)
            && (chain.triggering_location == u16::from(LOCATION_DECK)
                || (chain.triggering_location == u16::from(LOCATION_EXTRA) && u32::from(chain.triggering_position) & POS_FACEDOWN != 0))
        {
            self.release_relation(handler, chain.triggering_effect, chain.chain_id);
        }
        true
    }

    pub(crate) fn field_draw(&mut self, reason_effect: Option<EffectId>, reason: u32, reason_player: u8, playerid: u8, count: u16) {
        self.emplace_process(Draw {
            reason_effect,
            reason,
            reason_player,
            playerid,
            count,
            ..Draw::default()
        });
    }

    // Sets current.reason and sendto_param; scripts' send-to operations enter here.
    pub(crate) fn send_to(
        &mut self,
        targets: CardSet,
        reason_effect: Option<EffectId>,
        reason: u32,
        reason_player: u8,
        playerid: u8,
        destination: u8,
        sequence: u8,
        position: u8,
        ignore: bool,
    ) {
        if destination & LOCATION_ONFIELD != 0 {
            return;
        }
        for card in targets.iter() {
            let target = &mut self.cards[card];
            target.temp.reason = target.current.reason;
            target.temp.reason_effect = target.current.reason_effect;
            target.temp.reason_player = target.current.reason_player;
            target.current.reason = reason;
            target.current.reason_effect = reason_effect;
            target.current.reason_player = reason_player;
            let mut p = playerid;
            // Sent to the hand from the deck without a player: to the hand of the controller.
            if p == PLAYER_NONE
                && destination & LOCATION_HAND != 0
                && target.current.location & LOCATION_DECK != 0
                && target.current.controler == reason_player
            {
                p = reason_player;
            }
            if p == PLAYER_NONE {
                p = target.owner;
            }
            if destination == LOCATION_GRAVE && target.current.location == LOCATION_REMOVED {
                target.current.reason |= REASON_RETURN;
            }
            let position = if destination != LOCATION_REMOVED && !ignore {
                POS_FACEUP as u8
            } else if position == 0 {
                target.current.position as u8
            } else {
                position
            };
            target.sendto_param = SendtoParam {
                playerid: p,
                position,
                location: destination,
                sequence,
            };
        }
        self.emplace_process(SendTo {
            targets,
            reason_effect,
            reason,
            reason_player,
            ..SendTo::default()
        });
    }

    pub(crate) fn field_damage(
        &mut self,
        reason_effect: Option<EffectId>,
        reason: u32,
        reason_player: u8,
        reason_card: Option<CardId>,
        playerid: u8,
        amount: u32,
    ) {
        let (reason_effect, reason_card) = if reason & REASON_BATTLE != 0 {
            (None, reason_card)
        } else {
            (reason_effect, None)
        };
        self.emplace_process(Damage {
            step: 0,
            reason_player,
            playerid,
            amount,
            reason,
            reason_card,
            reason_effect,
        });
    }

    pub(crate) fn process_draw(&mut self, arg: &mut Draw) -> bool {
        let (reason_effect, reason, reason_player, playerid) = (arg.reason_effect, arg.reason, arg.reason_player, arg.playerid);
        match arg.step {
            0 => {
                if (reason & REASON_RULE == 0 && !self.field_is_player_can_draw(playerid)) || arg.count == 0 {
                    self.returns.set_i32(0, 0);
                    return true;
                }
                let mut drawn = Cards16::default();
                let mut public_count = 0usize;
                self.core.overdraw[playerid] = false;
                for _ in 0..arg.count {
                    let Some(&card) = self.player[playerid].list_main.last() else {
                        self.core.overdraw[playerid] = true;
                        break;
                    };
                    self.enable_field_effect(card, false);
                    self.cancel_field_effect(card);
                    self.player[playerid].list_main.pop();
                    if !self.core.current_chain.is_empty() {
                        self.core.just_sent_cards.insert(card);
                    }
                    let target = &mut self.cards[card];
                    target.previous.controler = target.current.controler;
                    target.previous.location = target.current.location;
                    target.previous.sequence = target.current.sequence;
                    target.previous.position = target.current.position;
                    target.previous.pzone = target.current.pzone;
                    target.current.controler = PLAYER_NONE;
                    target.current.reason_effect = reason_effect;
                    target.current.reason_player = reason_player;
                    target.current.reason = reason | REASON_DRAW;
                    target.current.location = 0;
                    self.add_card(playerid, card, LOCATION_HAND, 0, false);
                    self.enable_field_effect(card, true);
                    let public = self.is_affected_by_effect(card, EFFECT_PUBLIC).is_some();
                    if public {
                        public_count += 1;
                    }
                    self.cards[card].current.position = if public { POS_FACEUP } else { POS_FACEDOWN };
                    drawn.push(card);
                    self.reset(card, RESET_TOHAND, RESET_EVENT);
                }
                self.core.hint_timing[playerid] |= TIMING_DRAW + TIMING_TOHAND;
                self.adjust_instant();
                arg.count = drawn.len() as u16;
                arg.drawn_set = drawn.iter().copied().collect();
                if drawn.is_empty() {
                    return false;
                }
                if self.core.global_flag & GLOBALFLAG_DECK_REVERSE_CHECK != 0
                    && let Some(&top) = self.player[playerid].list_main.last()
                {
                    let (code, position) = (self.cards[top].data.code, self.cards[top].current.position);
                    if self.core.deck_reversed || position == POS_FACEUP_DEFENSE {
                        self.trace
                            .message(MSG_DECK_TOP)
                            .u8(playerid)
                            .u32(drawn.len() as u32)
                            .u32(code)
                            .u32(position);
                    }
                }
                let mut message = self.trace.message(MSG_DRAW);
                message.u8(playerid).u32(drawn.len() as u32);
                for &card in &drawn {
                    message.u32(self.cards[card].data.code).u32(self.cards[card].current.position);
                }
                drop(message);
                let drawn_set = arg.drawn_set;
                if self.core.deck_reversed && public_count < drawn.len() {
                    let mut message = self.trace.message(MSG_CONFIRM_CARDS);
                    message.u8(1 - playerid).u32(drawn_set.len() as u32);
                    for card in drawn_set.iter() {
                        let current = self.cards[card].current;
                        message
                            .u32(self.cards[card].data.code)
                            .u8(current.controler)
                            .u8(current.location)
                            .u32(current.sequence);
                    }
                    drop(message);
                    self.shuffle(playerid, LOCATION_HAND);
                }
                // A card drawn by a player other than its owner gets the client hint 67; control changes come with the
                // cards that cause them.
                unported!(
                    self,
                    drawn_set
                        .iter()
                        .filter(|&card| self.cards[card].owner != self.cards[card].current.controler)
                );
                for card in drawn_set.iter() {
                    for event in [EVENT_DRAW, EVENT_TO_HAND, EVENT_MOVE] {
                        self.raise_single_event(card, event, reason_effect, reason, reason_player, playerid, 0);
                    }
                }
                self.process_single_event();
                for event in [EVENT_DRAW, EVENT_TO_HAND, EVENT_MOVE] {
                    self.field_raise_event(
                        Some(drawn_set),
                        event,
                        reason_effect,
                        reason,
                        reason_player,
                        playerid,
                        u32::from(arg.count),
                    );
                }
                self.process_instant_event();
                false
            }
            1 => {
                std::mem::swap(&mut self.core.operated_set, &mut arg.drawn_set);
                self.returns.set_u32(0, u32::from(arg.count));
                true
            }
            _ => true,
        }
    }

    // field::refresh_location_info_instant: the zones EFFECT_DISABLE_FIELD and EFFECT_USE_EXTRA_*ZONE take away, which
    // come with the cards that register them.
    pub(crate) fn refresh_location_info_instant(&self) {
        for code in [EFFECT_DISABLE_FIELD, EFFECT_USE_EXTRA_MZONE, EFFECT_USE_EXTRA_SZONE] {
            unported!(self, self.filter_field_effect(code));
        }
    }

    // Equipping with is_step (Duel.Equip(..., true) and Duel.EquipComplete) comes with the cards that call it.
    // Unique cards (unique_code) come with the cards that have them.
    pub(crate) fn process_get_control(&mut self, arg: &mut GetControl) -> bool {
        let playerid = arg.playerid;
        match arg.step {
            0 => {
                arg.destroy_set.clear();
                for card in arg.targets {
                    self.filter_disable_related_cards(card);
                    let target = &self.cards[card];
                    let change = target.overlay_target.is_none()
                        && target.current.controler != playerid
                        && target.current.controler != PLAYER_NONE
                        && target.current.location == LOCATION_MZONE
                        && self.is_affected_by_effect(card, EFFECT_CANNOT_CHANGE_CONTROL).is_none()
                        && arg.reason_effect.is_none_or(|effect| self.is_affect_by_effect(card, Some(effect)))
                        && (self.is_flag(DUEL_TRAP_MONSTERS_NOT_USE_ZONE)
                            || self.get_type(card) & TYPE_TRAPMONSTER == 0
                            || self
                                .get_useable_count(Some(card), playerid, LOCATION_SZONE, playerid, LOCATION_REASON_CONTROL, 0xff)
                                .0
                                > 0);
                    if !change {
                        arg.targets.remove(card);
                    }
                }
                let fcount = self
                    .get_useable_count(None, playerid, LOCATION_MZONE, playerid, LOCATION_REASON_CONTROL, arg.zone)
                    .0;
                if fcount <= 0 {
                    std::mem::swap(&mut arg.destroy_set, &mut arg.targets);
                    arg.step = 5;
                    return false;
                }
                if arg.targets.len() as i32 > fcount {
                    self.core.select_cards = arg.targets.iter().collect();
                    self.trace.message(MSG_HINT).u8(HINT_SELECTMSG).u8(playerid).u64(502);
                    let ct = (arg.targets.len() as i32 - fcount) as u8;
                    self.emplace_process(SelectCard {
                        playerid,
                        min: ct,
                        max: ct,
                        ..SelectCard::default()
                    });
                } else {
                    arg.step = 1;
                }
                false
            }
            1 => {
                let returned = self.returned_cards();
                arg.destroy_set = arg.destroy_set.union(returned);
                arg.targets = arg.targets.difference(returned);
                false
            }
            2 => {
                arg.remaining = arg.targets;
                false
            }
            3 => {
                let Some(card) = arg.remaining.get_first() else {
                    self.adjust_instant();
                    arg.step = 4;
                    return false;
                };
                let move_player = if arg.chose_player == PLAYER_NONE {
                    playerid
                } else {
                    arg.chose_player
                };
                let position = self.cards[card].current.position as u8;
                let zone = arg.zone as u8;
                self.field_move_to_field(
                    card,
                    move_player,
                    playerid,
                    LOCATION_MZONE.into(),
                    position,
                    false,
                    zone,
                    false,
                    LOCATION_REASON_NONE,
                );
                false
            }
            4 => {
                let Some(card) = arg.remaining.get_first() else { return true };
                self.cards[card].set_status(STATUS_ATTACK_CANCELED, true);
                self.set_control(card, playerid, arg.reset_phase, arg.reset_count.into());
                self.reset(card, RESET_CONTROL, RESET_EVENT);
                self.filter_disable_related_cards(card);
                arg.remaining.remove(card);
                arg.step = 2;
                false
            }
            5 => {
                let reason_player = if arg.chose_player == PLAYER_NONE {
                    PLAYER_NONE
                } else {
                    self.core.reason_player.get()
                };
                for card in arg.targets {
                    if self.cards[card].current.location & LOCATION_ONFIELD == 0 {
                        arg.targets.remove(card);
                        continue;
                    }
                    for event in [EVENT_CONTROL_CHANGED, EVENT_MOVE] {
                        self.raise_single_event(card, event, arg.reason_effect, REASON_EFFECT, reason_player, playerid, 0);
                    }
                }
                if !arg.targets.is_empty() {
                    for event in [EVENT_CONTROL_CHANGED, EVENT_MOVE] {
                        self.field_raise_event(
                            Some(arg.targets),
                            event,
                            arg.reason_effect,
                            REASON_EFFECT,
                            reason_player,
                            playerid,
                            0,
                        );
                    }
                }
                self.process_single_event();
                self.process_instant_event();
                false
            }
            6 => {
                if !arg.destroy_set.is_empty() {
                    self.field_destroy(arg.destroy_set, None, REASON_RULE, PLAYER_NONE, PLAYER_NONE, LOCATION_GRAVE, 0);
                }
                false
            }
            _ => {
                self.core.operated_set = arg.targets;
                self.returns.set_i32(0, arg.targets.len() as i32);
                true
            }
        }
    }

    pub(crate) fn process_equip(&mut self, arg: &mut Equip) -> bool {
        let (equip_player, equip_card, target) = (arg.equip_player, arg.equip_card, arg.target);
        match arg.step {
            0 => {
                self.returns.set_i32(0, 0);
                if !self.is_affect_by_effect(equip_card, self.core.reason_effect.get()) || equip_card == target {
                    return true;
                }
                let mut to_grave = false;
                let current = self.cards[target].current;
                if current.location != LOCATION_MZONE || current.position & POS_FACEDOWN != 0 {
                    if self.is_flag(DUEL_EQUIP_NOT_SENT_IF_MISSING_TARGET) && self.cards[equip_card].current.location == LOCATION_MZONE {
                        return true;
                    }
                    to_grave = true;
                }
                if self.cards[equip_card].current.location != LOCATION_SZONE {
                    self.refresh_location_info_instant();
                    let count = self.get_useable_count(
                        Some(equip_card),
                        equip_player,
                        LOCATION_SZONE,
                        equip_player,
                        LOCATION_REASON_TOFIELD,
                        0xff,
                    );
                    if count.0 <= 0 {
                        to_grave = true;
                    }
                }
                if to_grave {
                    if self.cards[equip_card].current.location != LOCATION_GRAVE {
                        let reason = REASON_RULE;
                        self.send_to(
                            equip_card.into(),
                            None,
                            reason,
                            PLAYER_NONE,
                            PLAYER_NONE,
                            LOCATION_GRAVE,
                            0,
                            POS_FACEUP as u8,
                            false,
                        );
                    }
                    arg.step = 2;
                    return false;
                }
                if let Some(previous) = self.cards[equip_card].equiping_target {
                    self.card_targets.retain(|&link| link != (equip_card, previous));
                    self.card_unequip(equip_card);
                    self.enable_field_effect(equip_card, false);
                    return false;
                }
                if self.cards[equip_card].current.location == LOCATION_SZONE {
                    if arg.faceup && self.cards[equip_card].is_position(POS_FACEDOWN) {
                        self.change_position(equip_card, None, equip_player, POS_FACEUP, 0, false);
                    }
                    return false;
                }
                self.enable_field_effect(equip_card, false);
                self.cards[equip_card].current.reason_player = equip_player;
                let faceup = arg.faceup || self.cards[equip_card].is_position(POS_FACEUP);
                let position = (if faceup { POS_FACEUP } else { POS_FACEDOWN }) as u8;
                let destination = LOCATION_SZONE.into();
                self.field_move_to_field(
                    equip_card,
                    equip_player,
                    equip_player,
                    destination,
                    position,
                    false,
                    0xff,
                    false,
                    LOCATION_REASON_TOFIELD,
                );
                false
            }
            1 => {
                self.card_equip(equip_card, target);
                let data_type = self.cards[equip_card].data.card_type;
                if data_type & TYPE_EQUIP == 0 {
                    let (code, value) = if self.get_type(equip_card) & TYPE_TRAP != 0 {
                        (EFFECT_ADD_TYPE, TYPE_EQUIP)
                    } else if data_type & TYPE_UNION != 0 {
                        (EFFECT_CHANGE_TYPE, TYPE_EQUIP + TYPE_SPELL + TYPE_UNION)
                    } else if data_type & TYPE_TOKEN != 0 {
                        (EFFECT_CHANGE_TYPE, TYPE_EQUIP + TYPE_SPELL + TYPE_TOKEN)
                    } else {
                        (EFFECT_CHANGE_TYPE, TYPE_EQUIP + TYPE_SPELL)
                    };
                    let effect = self.new_effect(equip_card);
                    let peffect = &mut self.effects[effect];
                    peffect.handler = Some(equip_card);
                    peffect.effect_type = EFFECT_TYPE_SINGLE;
                    peffect.code = code;
                    peffect.value = Value::Int(value as i32);
                    peffect.flag[0] = EFFECT_FLAG_CANNOT_DISABLE;
                    peffect.reset_flag = RESET_EVENT + 0x17e_0000;
                    self.card_add_effect(equip_card, effect);
                }
                if !self.card_targets.contains(&(equip_card, target)) {
                    self.card_targets.push((equip_card, target));
                }
                unported!(self, arg.is_step.then_some(equip_card));
                if self.cards[equip_card].is_position(POS_FACEUP) {
                    self.enable_field_effect(equip_card, true);
                }
                self.adjust_disable_check_list();
                let (reason_effect, reason_player) = (self.core.reason_effect.get(), self.core.reason_player.get());
                let equips = Some(CardSet::from(equip_card));
                self.raise_single_event_with(target, equips, EVENT_EQUIP, reason_effect, 0, reason_player, PLAYER_NONE, 0);
                self.field_raise_event(equips, EVENT_EQUIP, reason_effect, 0, reason_player, PLAYER_NONE, 0);
                let card = &self.cards[target];
                let controler = card
                    .overlay_target
                    .map_or(card.current.controler, |overlay| self.cards[overlay].current.controler);
                self.core.hint_timing[controler] |= TIMING_EQUIP;
                self.process_single_event();
                self.process_instant_event();
                false
            }
            2 => {
                self.returns.set_i32(0, 1);
                true
            }
            _ => {
                self.returns.set_i32(0, 0);
                true
            }
        }
    }

    // Destinations other than the GY come from redirect effects, which destination_redirect keeps unported.
    pub(crate) fn process_discard_deck(&mut self, arg: &mut DiscardDeck) -> bool {
        let (playerid, count, reason) = (arg.playerid, usize::from(arg.count), arg.reason);
        let (reason_effect, reason_player) = (self.core.reason_effect.get(), self.core.reason_player.get());
        match arg.step {
            0 => {
                if self.is_player_affected_by_effect(playerid, EFFECT_CANNOT_DISCARD_DECK).is_some() {
                    self.core.operated_set.clear();
                    self.returns.set_i32(0, 0);
                    return true;
                }
                let top: Vec<CardId> = self.player[playerid].list_main.iter().rev().take(count).copied().collect();
                for card in top {
                    let target = &mut self.cards[card];
                    target.sendto_param.location = LOCATION_GRAVE;
                    target.current.reason_effect = reason_effect;
                    target.current.reason_player = reason_player;
                    target.current.reason = reason;
                    self.destination_redirect(card, LOCATION_GRAVE);
                }
                let list = &self.player[playerid].list_main;
                if self.core.global_flag & GLOBALFLAG_DECK_REVERSE_CHECK != 0
                    && list.len() > count
                    && let Some(&top) = list.iter().rev().nth(count)
                {
                    let (code, position) = (self.cards[top].data.code, self.cards[top].current.position);
                    if self.core.deck_reversed || position == POS_FACEUP_DEFENSE {
                        self.trace
                            .message(MSG_DECK_TOP)
                            .u8(playerid)
                            .u32(count as u32)
                            .u32(code)
                            .u32(position);
                    }
                }
                false
            }
            1 => {
                arg.discarded_set.clear();
                for _ in 0..count {
                    let Some(&card) = self.player[playerid].list_main.last() else {
                        break;
                    };
                    let destination = self.cards[card].sendto_param.location;
                    self.reset(card, RESET_TOGRAVE, RESET_EVENT);
                    let from = self.cards.get_info_location(card);
                    self.enable_field_effect(card, false);
                    self.cancel_field_effect(card);
                    self.player[playerid].list_main.pop();
                    if !self.core.current_chain.is_empty() {
                        self.core.just_sent_cards.insert(card);
                    }
                    let target = &mut self.cards[card];
                    target.previous.controler = target.current.controler;
                    target.previous.location = target.current.location;
                    target.previous.sequence = target.current.sequence;
                    target.previous.position = target.current.position;
                    target.previous.pzone = target.current.pzone;
                    target.current.controler = PLAYER_NONE;
                    target.current.location = 0;
                    let owner = target.owner;
                    self.add_card(owner, card, destination, 0, false);
                    self.enable_field_effect(card, true);
                    self.cards[card].current.position = POS_FACEUP;
                    let (code, to, card_reason) = (
                        self.cards[card].data.code,
                        self.cards.get_info_location(card),
                        self.cards[card].current.reason,
                    );
                    self.trace.message(MSG_MOVE).u32(code).loc(from).loc(to).u32(card_reason);
                    let current = self.cards[card].current;
                    for event in [EVENT_TO_GRAVE, EVENT_MOVE] {
                        self.raise_single_event(card, event, current.reason_effect, current.reason, current.reason_player, 0, 0);
                    }
                    arg.discarded_set.insert(card);
                }
                let discarded = Some(arg.discarded_set).filter(|set| !set.is_empty());
                if discarded.is_some() {
                    self.field_raise_event(discarded, EVENT_TO_GRAVE, reason_effect, reason, reason_player, 0, 0);
                }
                self.field_raise_event(Some(arg.discarded_set), EVENT_MOVE, reason_effect, reason, reason_player, 0, 0);
                self.process_single_event();
                self.process_instant_event();
                self.adjust_instant();
                false
            }
            2 => {
                self.core.operated_set = arg.discarded_set;
                self.returns.set_i32(0, arg.discarded_set.len() as i32);
                true
            }
            _ => true,
        }
    }

    pub(crate) fn process_damage(&mut self, arg: &mut Damage) -> bool {
        let (playerid, amount, reason, reason_card) = (arg.playerid, arg.amount, arg.reason, arg.reason_card);
        let reason_player = arg.reason_player;
        match arg.step {
            0 => {
                self.returns.set_u32(0, amount);
                if amount == 0 {
                    return true;
                }
                if reason & REASON_RDAMAGE == 0 {
                    unported!(self, self.filter_player_effect(playerid, EFFECT_REVERSE_DAMAGE, true));
                }
                unported!(self, self.filter_player_effect(playerid, EFFECT_REFLECT_DAMAGE, true));
                unported!(self, self.filter_player_effect(playerid, EFFECT_CHANGE_DAMAGE, true));
                false
            }
            1 => {
                self.core.hint_timing[playerid] |= TIMING_DAMAGE;
                self.player[playerid].lp -= amount as i32;
                self.trace.message(MSG_DAMAGE).u8(playerid).u32(amount);
                self.field_raise_event(
                    reason_card.map(CardSet::from),
                    EVENT_DAMAGE,
                    arg.reason_effect,
                    reason,
                    reason_player,
                    playerid,
                    amount,
                );
                if let Some(card) = reason_card.filter(|_| reason == REASON_BATTLE) {
                    if self.player[playerid].lp <= 0 && self.core.attack_target.is_none() {
                        unported!(self, self.is_affected_by_effect(card, EFFECT_MATCH_KILL));
                    }
                    self.raise_single_event(card, EVENT_BATTLE_DAMAGE, None, 0, reason_player, playerid, amount);
                    self.field_raise_event(Some(card.into()), EVENT_BATTLE_DAMAGE, None, 0, reason_player, playerid, amount);
                    self.process_single_event();
                }
                if self.player[playerid].lp < 0 {
                    unported!(self, self.is_player_affected_by_effect(playerid, EFFECT_CANNOT_LOSE_LP));
                }
                self.process_instant_event();
                false
            }
            2 => {
                self.returns.set_u32(0, amount);
                true
            }
            _ => true,
        }
    }

    pub(crate) fn field_recover(&mut self, reason_effect: Option<EffectId>, reason: u32, reason_player: u8, playerid: u8, amount: u32) {
        self.emplace_process(Recover {
            step: 0,
            reason_player,
            playerid,
            amount,
            reason,
            reason_effect,
        });
    }

    pub(crate) fn process_recover(&mut self, arg: &mut Recover) -> bool {
        let (playerid, amount, reason, reason_player) = (arg.playerid, arg.amount, arg.reason, arg.reason_player);
        match arg.step {
            0 => {
                self.returns.set_u32(0, amount);
                if amount == 0 {
                    return true;
                }
                if reason & REASON_RRECOVER == 0 {
                    unported!(self, self.filter_player_effect(playerid, EFFECT_REVERSE_RECOVER, true));
                }
                false
            }
            1 => {
                if reason & REASON_RDAMAGE != 0 {
                    arg.step = 2;
                }
                self.core.hint_timing[playerid] |= TIMING_RECOVER;
                self.player[playerid].lp += amount as i32;
                self.trace.message(MSG_RECOVER).u8(playerid).u32(amount);
                self.field_raise_event(None, EVENT_RECOVER, arg.reason_effect, reason, reason_player, playerid, amount);
                self.process_instant_event();
                false
            }
            2 => {
                self.returns.set_u32(0, amount);
                true
            }
            _ => true,
        }
    }

    // field::destroy: cards already confirmed for destruction are left out, unless a substitute canceled it.
    pub(crate) fn field_destroy(
        &mut self,
        targets: CardSet,
        reason_effect: Option<EffectId>,
        reason: u32,
        reason_player: u8,
        playerid: u8,
        destination: u8,
        sequence: u8,
    ) {
        let mut targets = targets;
        for card in targets {
            let canceled = self.core.destroy_canceled.contains(card);
            let target = &mut self.cards[card];
            if target.is_status(STATUS_DESTROY_CONFIRMED) && !canceled {
                targets.remove(card);
                continue;
            }
            target.temp.reason = target.current.reason;
            target.current.reason = reason;
            if reason_player != PLAYER_SELFDES {
                target.temp.reason_effect = target.current.reason_effect;
                target.temp.reason_player = target.current.reason_player;
                if reason_effect.is_some() {
                    target.current.reason_effect = reason_effect;
                }
                target.current.reason_player = reason_player;
            }
            let destination = if destination & (LOCATION_HAND | LOCATION_DECK | LOCATION_REMOVED) == 0 {
                LOCATION_GRAVE
            } else {
                destination
            };
            let mut p = playerid;
            if p == PLAYER_NONE || destination & (LOCATION_GRAVE | LOCATION_REMOVED) != 0 {
                p = target.owner;
            }
            target.set_status(STATUS_DESTROY_CONFIRMED, true);
            target.sendto_param = SendtoParam {
                playerid: p,
                position: POS_FACEUP as u8,
                location: destination,
                sequence,
            };
        }
        self.emplace_process(Destroy {
            step: 0,
            reason_player,
            reason,
            targets,
            reason_effect,
        });
    }

    // The core shares `targets` as a group with DestroyReplace, which steps 1 and 11 run inline, and with the
    // BattleCommand that started step 10; while no card substitutes destruction, step 10 keeps every card, so the
    // copies stay equal.
    pub(crate) fn process_destroy(&mut self, arg: &mut Destroy) -> bool {
        let (reason_effect, reason, reason_player) = (arg.reason_effect, arg.reason, arg.reason_player);
        match arg.step {
            0 => {
                let (mut indestructable, mut extra) = (CardSet::default(), CardSet::default());
                let mut indestructable_effects = EffectSet::default();
                for card in arg.targets {
                    if !self.is_destructable(card) {
                        indestructable.insert(card);
                        continue;
                    }
                    let current = self.cards[card].current;
                    if current.reason & (REASON_RULE | REASON_COST) == 0 {
                        let destructable = if current.reason_effect.is_none() || self.is_affect_by_effect(card, current.reason_effect) {
                            let effect = self.check_indestructable_by_effect(card, current.reason_effect, current.reason_player);
                            if let Some(effect) = effect
                                && reason_player != PLAYER_SELFDES
                                && !indestructable_effects.contains(&effect)
                            {
                                indestructable_effects.push(effect);
                            }
                            effect.is_none()
                        } else {
                            false
                        };
                        if !destructable {
                            indestructable.insert(card);
                            continue;
                        }
                    }
                    let codes = [EFFECT_INDESTRUCTABLE, EFFECT_INDESTRUCTABLE_COUNT];
                    unported!(self, codes.into_iter().flat_map(|code| self.filter_effect(card, code, true)));
                    let substitutes = self.destroy_substitutes(card);
                    if !substitutes.is_empty() {
                        extra.merge(substitutes);
                        self.restore_reason(card);
                        self.core.destroy_canceled.insert(card);
                        arg.targets.remove(card);
                    }
                }
                for card in indestructable {
                    self.restore_reason(card);
                    self.cards[card].set_status(STATUS_DESTROY_CONFIRMED, false);
                    arg.targets.remove(card);
                }
                for rep in extra {
                    if arg.targets.contains(rep) {
                        continue;
                    }
                    let target = &mut self.cards[rep];
                    target.temp.reason = target.current.reason;
                    target.temp.reason_effect = target.current.reason_effect;
                    target.temp.reason_player = target.current.reason_player;
                    target.current.reason = REASON_EFFECT | REASON_DESTROY | REASON_REPLACE;
                    target.current.reason_effect = None;
                    target.current.reason_player = target.current.controler;
                    target.sendto_param = SendtoParam {
                        playerid: target.owner,
                        position: POS_FACEUP as u8,
                        location: LOCATION_GRAVE,
                        sequence: 0,
                    };
                    arg.targets.insert(rep);
                }
                // The core keeps them in a set ordered by id.
                indestructable_effects.sort_by_key(|&effect| self.effects.id(effect));
                for effect in indestructable_effects {
                    self.dec_count(effect, PLAYER_NONE);
                    let code = self.cards[self.effects[effect].owner].data.code;
                    self.trace.message(MSG_HINT).u8(HINT_CARD).u8(0).u64(u64::from(code));
                }
                // operation_replace(EFFECT_DESTROY_REPLACE, 5, targets)
                unported!(self, self.field_effect_range(Container::Continuous, EFFECT_DESTROY_REPLACE));
                false
            }
            1 => {
                for card in arg.targets {
                    if self.cards[card].current.location & (LOCATION_GRAVE | LOCATION_REMOVED) != 0 {
                        self.restore_reason(card);
                        self.cards[card].set_status(STATUS_DESTROY_CONFIRMED, false);
                        arg.targets.remove(card);
                        continue;
                    }
                    self.returns.set_i32(0, 0);
                    unported!(self, self.filter_single_continuous_effect(card, EFFECT_DESTROY_REPLACE));
                }
                false
            }
            2 => {
                for card in std::mem::take(&mut self.core.destroy_canceled) {
                    self.cards[card].set_status(STATUS_DESTROY_CONFIRMED, false);
                }
                false
            }
            3 => {
                if arg.targets.is_empty() {
                    self.returns.set_i32(0, 0);
                    self.core.operated_set.clear();
                    return true;
                }
                let mut cv: Cards16 = arg.targets.iter().collect();
                cv.sort_by(|&c1, &c2| self.card_operation_sort(c1, c2));
                for card in cv {
                    if self.cards[card].current.location & (LOCATION_GRAVE | LOCATION_REMOVED) != 0 {
                        self.restore_reason(card);
                        arg.targets.remove(card);
                        continue;
                    }
                    self.cards[card].current.reason |= REASON_DESTROY;
                    let target = &self.cards[card];
                    let current = target.current;
                    let controler = target
                        .overlay_target
                        .map_or(current.controler, |overlay| self.cards[overlay].current.controler);
                    self.core.hint_timing[controler] |= TIMING_DESTROY;
                    self.raise_single_event(
                        card,
                        EVENT_DESTROY,
                        current.reason_effect,
                        current.reason,
                        current.reason_player,
                        0,
                        0,
                    );
                }
                self.adjust_instant();
                self.process_single_event();
                self.field_raise_event(Some(arg.targets), EVENT_DESTROY, reason_effect, reason, reason_player, 0, 0);
                self.process_instant_event();
                false
            }
            4 => {
                for card in arg.targets.iter() {
                    self.cards[card].set_status(STATUS_DESTROY_CONFIRMED, false);
                    let param = self.cards[card].sendto_param;
                    let destination = if param.location == 0 { LOCATION_GRAVE } else { param.location };
                    let destination = if (destination == LOCATION_HAND && !self.is_capable_send_to_hand(card, reason_player))
                        || (destination == LOCATION_DECK && !self.is_capable_send_to_deck(card, reason_player))
                        || (destination == LOCATION_REMOVED && !self.is_removeable(card, reason_player, param.position, reason))
                    {
                        LOCATION_GRAVE
                    } else {
                        destination
                    };
                    self.cards[card].sendto_param.location = destination;
                }
                // operation_replace(EFFECT_SEND_REPLACE, 5, targets)
                unported!(self, self.field_effect_range(Container::Continuous, EFFECT_SEND_REPLACE));
                let (targets, reason) = (arg.targets, reason | REASON_DESTROY);
                self.emplace_process(SendTo {
                    step: 1,
                    targets,
                    reason_effect,
                    reason,
                    reason_player,
                    extra_args: None,
                });
                false
            }
            5 => {
                let replaced: CardSet = arg
                    .targets
                    .iter()
                    .filter(|&card| self.cards[card].current.reason & REASON_REPLACE != 0)
                    .collect();
                self.core.operated_set = arg.targets.difference(replaced);
                self.returns.set_i32(0, self.core.operated_set.len() as i32);
                true
            }
            10 => {
                let targets = arg.targets;
                for card in targets.iter() {
                    if !self.is_destructable(card) {
                        self.restore_reason(card);
                        self.cards[card].set_status(STATUS_DESTROY_CONFIRMED, false);
                        arg.targets.remove(card);
                        continue;
                    }
                    let codes = [EFFECT_INDESTRUCTABLE, EFFECT_INDESTRUCTABLE_COUNT];
                    unported!(self, codes.into_iter().flat_map(|code| self.filter_effect(card, code, true)));
                    let substitutes = self.destroy_substitutes(card);
                    if !substitutes.is_empty() {
                        self.core.battle_destroy_rep.merge(substitutes);
                        self.restore_reason(card);
                        self.core.destroy_canceled.insert(card);
                        arg.targets.remove(card);
                    }
                }
                if !arg.targets.is_empty() {
                    // operation_replace(EFFECT_DESTROY_REPLACE, 12, targets)
                    unported!(self, self.field_effect_range(Container::Continuous, EFFECT_DESTROY_REPLACE));
                }
                false
            }
            11 => {
                let targets = arg.targets;
                for card in targets.iter() {
                    if self.cards[card].current.location & (LOCATION_GRAVE | LOCATION_REMOVED) != 0 {
                        self.restore_reason(card);
                        self.cards[card].set_status(STATUS_DESTROY_CONFIRMED, false);
                        arg.targets.remove(card);
                        continue;
                    }
                    self.returns.set_i32(0, 0);
                    unported!(
                        self,
                        self.field_effects
                            .registry
                            .equal_range(Container::Single, Some(card), EFFECT_DESTROY_REPLACE)
                    );
                }
                self.core.battle_destroyed = arg.targets;
                false
            }
            12 => {
                for card in std::mem::take(&mut self.core.destroy_canceled) {
                    self.cards[card].set_status(STATUS_DESTROY_CONFIRMED, false);
                }
                true
            }
            _ => true,
        }
    }

    // Steps 7 and 8 redirect Crystal Beasts through EFFECT_TO_GRAVE_REDIRECT_CB (phase 2).
    pub(crate) fn process_send_to(&mut self, arg: &mut SendTo) -> bool {
        let (reason_effect, reason, reason_player) = (arg.reason_effect, arg.reason, arg.reason_player);
        match arg.step {
            0 => {
                let targets = arg.targets;
                for card in targets.iter() {
                    let target = &self.cards[card];
                    let destination = target.sendto_param.location;
                    let rejected = reason & REASON_RULE == 0
                        && (target.get_status(STATUS_SUMMONING | STATUS_SPSUMMON_STEP)
                            || (target.current.reason & (REASON_COST | REASON_SUMMON | REASON_MATERIAL) == 0
                                && !self.is_affect_by_effect(card, target.current.reason_effect))
                            || (destination == LOCATION_HAND && !self.is_capable_send_to_hand(card, self.core.reason_player.get()))
                            || (destination == LOCATION_DECK && !self.is_capable_send_to_deck(card, self.core.reason_player.get()))
                            || (destination == LOCATION_REMOVED
                                && !self.is_removeable(card, self.core.reason_player.get(), target.sendto_param.position, reason))
                            || (destination == LOCATION_GRAVE
                                && !self.is_capable_send_to_grave(card, self.core.reason_player.get(), reason))
                            || (destination == LOCATION_EXTRA && !self.is_capable_send_to_extra(card, self.core.reason_player.get())));
                    if rejected {
                        self.restore_reason(card);
                        arg.targets.remove(card);
                    }
                }
                if reason & REASON_RULE != 0 {
                    return false;
                }
                // operation_replace(EFFECT_SEND_REPLACE, 5, targets)
                unported!(self, self.field_effect_range(Container::Continuous, EFFECT_SEND_REPLACE));
                false
            }
            1 => {
                let targets = arg.targets;
                for card in targets.iter() {
                    let target = &self.cards[card];
                    let (playerid, destination) = (target.sendto_param.playerid, target.sendto_param.location);
                    if target.current.location == destination && target.current.controler == playerid {
                        self.restore_reason(card);
                        arg.targets.remove(card);
                        continue;
                    }
                    if target.current.reason & REASON_RULE == 0 {
                        self.returns.set_i32(0, 0);
                        unported!(
                            self,
                            self.field_effects
                                .registry
                                .equal_range(Container::Single, Some(card), EFFECT_SEND_REPLACE)
                        );
                    }
                }
                false
            }
            2 => {
                if arg.targets.is_empty() {
                    self.returns.set_i32(0, 0);
                    self.core.operated_set.clear();
                    return true;
                }
                // Phase 2: cards leaving the field also snapshot their previous code, type, stats and set codes here.
                let mut leave_p = CardSet::default();
                let targets = arg.targets;
                for card in targets.iter() {
                    if !self.is_leaving_field(card) {
                        continue;
                    }
                    let current = self.cards[card].current;
                    self.raise_single_event(
                        card,
                        EVENT_LEAVE_FIELD_P,
                        current.reason_effect,
                        current.reason,
                        current.reason_player,
                        0,
                        0,
                    );
                    leave_p.insert(card);
                }
                if !leave_p.is_empty() {
                    self.field_raise_event(Some(leave_p), EVENT_LEAVE_FIELD_P, reason_effect, reason, reason_player, 0, 0);
                }
                self.process_single_event();
                self.process_instant_event();
                false
            }
            3 => {
                for card in arg.targets.iter() {
                    self.enable_field_effect(card, false);
                }
                self.adjust_disable_check_list();
                for card in arg.targets.iter() {
                    let destination = self.cards[card].sendto_param.location;
                    let check_cb =
                        destination & LOCATION_GRAVE != 0 && self.is_affected_by_effect(card, EFFECT_TO_GRAVE_REDIRECT_CB).is_some();
                    let mut destination = destination;
                    let redirect = if self.is_leaving_field(card) {
                        self.leave_field_redirect(card)
                    } else {
                        0
                    };
                    if redirect != 0 {
                        let target = &mut self.cards[card];
                        target.current.reason = (target.current.reason & !REASON_TEMPORARY) | REASON_REDIRECT;
                        target.sendto_param.location = redirect as u8;
                        target.sendto_param.sequence = (redirect >> 16) as u8;
                        destination = redirect as u8;
                        if destination == LOCATION_REMOVED {
                            let position = &mut target.sendto_param.position;
                            for (facedown, faceup) in [(POS_FACEDOWN_ATTACK, POS_FACEUP_ATTACK), (POS_FACEDOWN_DEFENSE, POS_FACEUP_DEFENSE)]
                            {
                                if u32::from(*position) & facedown != 0 {
                                    *position = ((u32::from(*position) & !facedown) | faceup) as u8;
                                }
                            }
                        }
                    }
                    let redirect = self.destination_redirect(card, destination);
                    let target = &mut self.cards[card];
                    if redirect != 0 && u32::from(target.current.location) != redirect & 0xffff {
                        target.current.reason |= REASON_REDIRECT;
                        target.sendto_param.location = redirect as u8;
                        target.sendto_param.sequence = (redirect >> 16) as u8;
                    }
                    if check_cb {
                        target.sendto_param.playerid |= 1 << 4;
                    }
                }
                false
            }
            4 => {
                let mut cv: Cards16 = arg.targets.iter().collect();
                if cv.len() > 1 {
                    cv.sort_by(|&c1, &c2| self.card_operation_sort(c1, c2));
                }
                if self.core.global_flag & GLOBALFLAG_DECK_REVERSE_CHECK != 0 {
                    self.check_deck_top_before_send(&cv);
                }
                arg.extra_args = Some(Box::new(SendToArgs {
                    cv,
                    ..SendToArgs::default()
                }));
                false
            }
            5 => {
                let Some(param) = &arg.extra_args else { return true };
                let Some(&card) = param.cv.get(param.cvit) else {
                    arg.step = 8;
                    return false;
                };
                if self.cards[card].sendto_param.playerid >> 4 != 0 {
                    unported!(self, self.is_affected_by_effect(card, EFFECT_TO_GRAVE_REDIRECT_CB));
                }
                self.enable_field_effect(card, false);
                // Tokens, which only card effects create, leave play here without moving (phase 2).
                self.returns.set_i32(0, 0);
                false
            }
            6 => {
                if self.returns.at_i32(0) != 0 {
                    return false;
                }
                let Some(param) = &mut arg.extra_args else { return true };
                let Some(&card) = param.cv.get(param.cvit) else { return true };
                let target = &self.cards[card];
                let oloc = target.current.location;
                let (playerid, destination, sequence) = (
                    target.sendto_param.playerid & 0x7,
                    target.sendto_param.location,
                    target.sendto_param.sequence,
                );
                let control_player = target
                    .overlay_target
                    .map_or(target.current.controler, |overlay| self.cards[overlay].current.controler);
                let timing = match destination {
                    LOCATION_GRAVE => TIMING_TOGRAVE,
                    LOCATION_HAND => TIMING_TOHAND,
                    LOCATION_DECK => TIMING_TODECK,
                    LOCATION_REMOVED => TIMING_REMOVE,
                    _ => 0,
                };
                self.core.hint_timing[control_player] |= timing;
                if destination == LOCATION_HAND || destination == LOCATION_DECK {
                    self.cards[card].set_status(STATUS_PROC_COMPLETE, false);
                }
                let current = self.cards[card].current;
                if current.controler != playerid || current.location != destination {
                    let from = self.cards.get_info_location(card);
                    if let Some(overlay_target) = self.cards[card].overlay_target {
                        param.detach.insert(overlay_target);
                        self.xyz_remove(overlay_target, card);
                    }
                    self.move_card(playerid, card, destination, sequence, false);
                    self.cards[card].current.position = u32::from(self.cards[card].sendto_param.position);
                    let (code, to, reason) = (
                        self.cards[card].data.code,
                        self.cards.get_info_location(card),
                        self.cards[card].current.reason,
                    );
                    self.trace.message(MSG_MOVE).u32(code).loc(from).loc(to).u32(reason);
                }
                let current = self.cards[card].current;
                if current.location == LOCATION_DECK && (self.core.deck_reversed || current.position == POS_FACEUP_DEFENSE) {
                    param.check_decktop_visibility[control_player] = true;
                }
                self.cards[card].set_status(STATUS_LEAVE_CONFIRMED, false);
                if self.cards[card].status & (STATUS_SUMMON_DISABLED | STATUS_ACTIVATE_DISABLED) != 0 {
                    self.cards[card].set_status(STATUS_SUMMON_DISABLED | STATUS_ACTIVATE_DISABLED, false);
                    self.cards[card].previous.location = 0;
                } else if oloc & LOCATION_ONFIELD != 0 {
                    self.reset(card, RESET_LEAVE, RESET_EVENT);
                    param.leave_field.insert(card);
                } else if oloc == LOCATION_GRAVE {
                    param.leave_grave.insert(card);
                }
                if self.cards[card].previous.location == LOCATION_OVERLAY {
                    self.cards[card].previous.controler = control_player;
                }
                param.cvit += 1;
                arg.step = 4;
                false
            }
            9 => {
                let Some(param) = arg.extra_args.take() else { return true };
                if self.core.global_flag & GLOBALFLAG_DECK_REVERSE_CHECK != 0 {
                    for p in 0..2 {
                        if !param.check_decktop_visibility[p] {
                            continue;
                        }
                        if let Some(&top) = self.player[p].list_main.last() {
                            let (code, position) = (self.cards[top].data.code, self.cards[top].current.position);
                            if self.core.deck_reversed || position == POS_FACEUP_DEFENSE {
                                self.trace.message(MSG_DECK_TOP).u8(p).u32(0).u32(code).u32(position);
                            }
                        }
                    }
                }
                for card in arg.targets.iter() {
                    if self.cards[card].data.card_type & TYPE_TOKEN == 0 {
                        self.enable_field_effect(card, true);
                    }
                    let new_location = self.cards[card].current.location;
                    let removed = new_location == LOCATION_REMOVED
                        || (self.cards[card].data.card_type & TYPE_TOKEN != 0
                            && self.cards[card].sendto_param.location == LOCATION_REMOVED);
                    let reset = match new_location {
                        LOCATION_HAND => RESET_TOHAND,
                        LOCATION_DECK | LOCATION_EXTRA => RESET_TODECK,
                        LOCATION_GRAVE => RESET_TOGRAVE,
                        _ => 0,
                    };
                    if reset != 0 {
                        self.reset(card, reset, RESET_EVENT);
                    }
                    if removed {
                        let temporary = self.cards[card].current.reason & REASON_TEMPORARY != 0;
                        self.reset(card, if temporary { RESET_TEMP_REMOVE } else { RESET_REMOVE }, RESET_EVENT);
                    }
                    self.refresh_disable_status(card);
                }
                for (cards, event) in [(param.leave_field, EVENT_LEAVE_FIELD), (param.leave_grave, EVENT_LEAVE_GRAVE)] {
                    for card in cards.iter() {
                        let current = self.cards[card].current;
                        self.raise_single_event(card, event, current.reason_effect, current.reason, current.reason_player, 0, 0);
                    }
                }
                let detach = if self.core.global_flag & GLOBALFLAG_DETACH_EVENT != 0 {
                    param.detach
                } else {
                    CardSet::default()
                };
                for card in detach.iter() {
                    if self.cards[card].current.location == LOCATION_MZONE {
                        self.raise_single_event(card, EVENT_DETACH_MATERIAL, reason_effect, reason, reason_player, 0, 0);
                    }
                }
                self.process_single_event();
                let events = [
                    (param.leave_field, EVENT_LEAVE_FIELD),
                    (param.leave_grave, EVENT_LEAVE_GRAVE),
                    (detach, EVENT_DETACH_MATERIAL),
                ];
                for (cards, event) in events {
                    if !cards.is_empty() {
                        self.field_raise_event(Some(cards), event, reason_effect, reason, reason_player, 0, 0);
                    }
                }
                self.process_instant_event();
                self.adjust_instant();
                false
            }
            10 => {
                let mut sets = [CardSet::default(); 7];
                let [to_hand, to_deck, to_grave, removed, discarded, released, destroyed] = &mut sets;
                let (mut overlays, mut equipings) = (CardSet::default(), CardSet::default());
                for card in arg.targets.iter() {
                    self.card_unequip(card);
                    for equip in self.equiping_cards(card) {
                        self.card_unequip(equip);
                        if self.cards[equip].current.location == LOCATION_SZONE {
                            equipings.insert(equip);
                        }
                    }
                    self.clear_card_target(card);
                    let target = &self.cards[card];
                    let (current, data_type, destination) = (target.current, target.data.card_type, target.sendto_param.location);
                    let raise = |set: &mut CardSet, event: u32, duel: &mut Self| {
                        set.insert(card);
                        duel.raise_single_event(card, event, current.reason_effect, current.reason, current.reason_player, 0, 0);
                    };
                    if current.location == LOCATION_HAND {
                        if target.owner != current.controler {
                            self.add_hint_effect(card, 67, RESET_EVENT + 0x1fe_0000);
                        }
                        // Added to the hand by a resolving effect.
                        if !self.core.current_chain.is_empty() {
                            let reset = RESET_EVENT
                                | RESET_TOFIELD
                                | RESET_LEAVE
                                | RESET_TODECK
                                | RESET_TOHAND
                                | RESET_TEMP_REMOVE
                                | RESET_REMOVE
                                | RESET_TOGRAVE
                                | RESET_TURN_SET
                                | RESET_CHAIN;
                            self.add_hint_effect(card, 225, reset);
                        }
                        raise(to_hand, EVENT_TO_HAND, self);
                    }
                    if current.location == LOCATION_DECK || current.location == LOCATION_EXTRA {
                        raise(to_deck, EVENT_TO_DECK, self);
                    }
                    if current.location == LOCATION_GRAVE && current.reason & REASON_RETURN == 0 {
                        raise(to_grave, EVENT_TO_GRAVE, self);
                    }
                    if current.location == LOCATION_REMOVED || (data_type & TYPE_TOKEN != 0 && destination == LOCATION_REMOVED) {
                        raise(removed, EVENT_REMOVE, self);
                    }
                    if current.reason & REASON_DISCARD != 0 {
                        raise(discarded, EVENT_DISCARD, self);
                    }
                    if current.reason & REASON_RELEASE != 0 {
                        raise(released, EVENT_RELEASE, self);
                    }
                    if current.reason & REASON_DESTROY != 0 && current.reason & REASON_BATTLE == 0 {
                        raise(destroyed, EVENT_DESTROYED, self);
                    }
                    overlays.merge(self.xyz_materials(card));
                    self.raise_single_event(card, EVENT_MOVE, current.reason_effect, current.reason, current.reason_player, 0, 0);
                }
                let events = [
                    EVENT_TO_HAND,
                    EVENT_TO_DECK,
                    EVENT_TO_GRAVE,
                    EVENT_REMOVE,
                    EVENT_DISCARD,
                    EVENT_RELEASE,
                    EVENT_DESTROYED,
                ];
                for (cards, event) in sets.into_iter().zip(events) {
                    if !cards.is_empty() {
                        self.field_raise_event(Some(cards), event, reason_effect, reason, reason_player, 0, 0);
                    }
                }
                self.field_raise_event(Some(arg.targets), EVENT_MOVE, reason_effect, reason, reason_player, 0, 0);
                self.process_single_event();
                self.process_instant_event();
                if !equipings.is_empty() {
                    let reason = REASON_RULE + REASON_LOST_TARGET;
                    self.field_destroy(equipings, None, reason, PLAYER_NONE, PLAYER_NONE, LOCATION_GRAVE, 0);
                }
                if !overlays.is_empty() {
                    let reason = REASON_RULE + REASON_LOST_TARGET;
                    self.send_to(
                        overlays,
                        None,
                        reason,
                        PLAYER_NONE,
                        PLAYER_NONE,
                        LOCATION_GRAVE,
                        0,
                        POS_FACEUP as u8,
                        false,
                    );
                }
                self.adjust_instant();
                false
            }
            11 => {
                self.core.operated_set = arg.targets;
                self.returns.set_i32(0, arg.targets.len() as i32);
                true
            }
            _ => true,
        }
    }

    fn is_leaving_field(&self, card: CardId) -> bool {
        let target = &self.cards[card];
        target.current.location & LOCATION_ONFIELD != 0
            && !target.is_status(STATUS_SUMMON_DISABLED)
            && !target.is_status(STATUS_ACTIVATE_DISABLED)
    }

    // The handlers of the card's EFFECT_DESTROY_SUBSTITUTE effects whose value (e, re, r, rp) accepts its destruction.
    fn destroy_substitutes(&self, card: CardId) -> CardSet {
        let current = self.cards[card].current;
        self.filter_effect(card, EFFECT_DESTROY_SUBSTITUTE, true)
            .into_iter()
            .filter(|&effect| match self.effects[effect].value {
                Value::Int(value) => value != 0,
                Value::Reason(value) => value(self, effect, current.reason_effect, current.reason, current.reason_player),
                _ => {
                    debug_assert!(false, "a value read with the wrong parameters");
                    false
                }
            })
            .map(|effect| self.get_handler(effect))
            .collect()
    }

    fn restore_reason(&mut self, card: CardId) {
        let target = &mut self.cards[card];
        target.current.reason = target.temp.reason;
        target.current.reason_player = target.temp.reason_player;
        target.current.reason_effect = target.temp.reason_effect;
    }

    // An effect of code 0 that only shows a hint (`description`) on the card until it resets.
    pub(crate) fn add_hint_effect(&mut self, card: CardId, description: u64, reset_flag: u32) {
        self.add_status_effect(card, 0, description, reset_flag);
    }

    // A single effect of `code` the core creates itself, shown as a hint on the card until it resets.
    pub(crate) fn add_status_effect(&mut self, card: CardId, code: u32, description: u64, reset_flag: u32) {
        let effect = self.new_effect(card);
        let peffect = &mut self.effects[effect];
        peffect.code = code;
        peffect.effect_type = EFFECT_TYPE_SINGLE;
        peffect.flag[0] = EFFECT_FLAG_CANNOT_DISABLE | EFFECT_FLAG_CLIENT_HINT;
        peffect.description = description;
        peffect.reset_flag = reset_flag;
        self.card_add_effect(card, effect);
    }

    pub(crate) fn process_spell_set(&mut self, arg: &mut SpellSet) -> bool {
        let (setplayer, toplayer, target) = (arg.setplayer, arg.toplayer, arg.target);
        match arg.step {
            0 => {
                let data_type = self.cards[target].data.card_type;
                if (data_type & TYPE_FIELD == 0
                    && self
                        .get_useable_count(Some(target), toplayer, LOCATION_SZONE, setplayer, LOCATION_REASON_TOFIELD, 0xff)
                        .0
                        <= 0)
                    || (data_type & TYPE_MONSTER != 0 && self.is_affected_by_effect(target, EFFECT_MONSTER_SSET).is_none())
                    || self.cards[target].current.location == LOCATION_SZONE
                    || !self.is_player_can_sset(setplayer, target)
                    || self.is_affected_by_effect(target, EFFECT_CANNOT_SSET).is_some()
                {
                    return true;
                }
                unported!(self, self.filter_effect(target, EFFECT_SSET_COST, true));
                false
            }
            1 => {
                self.enable_field_effect(target, false);
                let zone = if self.cards[target].data.card_type & TYPE_FIELD != 0 {
                    1 << 5
                } else {
                    0xff
                };
                self.field_move_to_field(
                    target,
                    setplayer,
                    toplayer,
                    LOCATION_SZONE.into(),
                    POS_FACEDOWN as u8,
                    false,
                    zone,
                    false,
                    LOCATION_REASON_NONE,
                );
                false
            }
            2 => {
                self.core.phase_action = true;
                self.cards[target].set_status(STATUS_SET_TURN, true);
                unported!(self, (self.cards[target].data.card_type & TYPE_MONSTER != 0).then_some(target));
                let (code, location) = (self.cards[target].data.code, self.cards.get_info_location(target));
                self.trace.message(MSG_SET).u32(code).loc(location);
                self.adjust_instant();
                self.field_raise_event(Some(target.into()), EVENT_SSET, arg.reason_effect, 0, setplayer, setplayer, 0);
                self.process_instant_event();
                if self.core.current_chain.is_empty() {
                    self.adjust_all();
                    self.core.hint_timing[setplayer] |= TIMING_SSET;
                    self.emplace_process(PointEvent::default());
                }
                true
            }
            _ => true,
        }
    }

    // field::special_summon: Duel.SpecialSummon's cards in one SpSummon. EFFECT_DEVINE_LIGHT and
    // EFFECT_FORCE_SPSUMMON_POSITION come with the cards that register them.
    pub(crate) fn field_special_summon(
        &mut self,
        targets: CardSet,
        sumtype: u32,
        sumplayer: u8,
        playerid: u8,
        nocheck: bool,
        nolimit: bool,
        positions: u8,
        zone: u32,
    ) {
        unported!(
            self,
            self.is_player_affected_by_effect(sumplayer, EFFECT_DEVINE_LIGHT)
                .filter(|_| u32::from(positions) & POS_FACEDOWN != 0)
        );
        unported!(self, self.filter_player_effect(sumplayer, EFFECT_FORCE_SPSUMMON_POSITION, true));
        let (reason_effect, reason_player) = (self.core.reason_effect.get(), self.core.reason_player.get());
        for card in targets {
            let target = &mut self.cards[card];
            target.temp.reason = target.current.reason;
            target.temp.reason_effect = target.current.reason_effect;
            target.temp.reason_player = target.current.reason_player;
            target.summon.summon_type = (sumtype & 0xf00_ffff) | SUMMON_TYPE_SPECIAL;
            target.summon.location = target.current.location;
            target.summon.sequence = target.current.sequence;
            target.summon.pzone = target.current.pzone;
            target.summon.player = sumplayer;
            target.current.reason = REASON_SPSUMMON;
            target.current.reason_effect = reason_effect;
            target.current.reason_player = reason_player;
            target.spsummon_param =
                (u32::from(playerid) << 24) + (u32::from(nocheck) << 16) + (u32::from(nolimit) << 8) + u32::from(positions);
        }
        self.emplace_process(SpSummon {
            step: 0,
            reason_player,
            zone,
            reason_effect,
            targets,
        });
    }

    // EFFECT_FORCE_SPSUMMON_POSITION comes with the cards that register it.
    pub(crate) fn field_special_summon_step(
        &mut self,
        target: CardId,
        sumtype: u32,
        sumplayer: u8,
        playerid: u8,
        nocheck: bool,
        nolimit: bool,
        positions: u8,
        zone: u32,
    ) {
        let mut positions = positions;
        if u32::from(positions) & POS_FACEDOWN != 0 && self.is_player_affected_by_effect(sumplayer, EFFECT_DEVINE_LIGHT).is_some() {
            positions = (positions & POS_FACEUP as u8) | ((positions & POS_FACEDOWN as u8) >> 1);
        }
        if u32::from(positions) & POS_FACEUP != 0 && self.check_unique_onfield(target, playerid, LOCATION_MZONE).is_some() {
            positions &= !(POS_FACEUP as u8);
        }
        unported!(self, self.filter_player_effect(sumplayer, EFFECT_FORCE_SPSUMMON_POSITION, true));
        let (reason_effect, reason_player) = (self.core.reason_effect.get(), self.core.reason_player.get());
        let card = &mut self.cards[target];
        card.temp.reason = card.current.reason;
        card.temp.reason_effect = card.current.reason_effect;
        card.temp.reason_player = card.current.reason_player;
        card.summon.summon_type = (sumtype & 0xf00_ffff) | SUMMON_TYPE_SPECIAL;
        card.summon.location = card.current.location;
        card.summon.sequence = card.current.sequence;
        card.summon.pzone = card.current.pzone;
        card.summon.player = sumplayer;
        card.current.reason = REASON_SPSUMMON;
        card.current.reason_effect = reason_effect;
        card.current.reason_player = reason_player;
        card.spsummon_param = (u32::from(playerid) << 24) + (u32::from(nocheck) << 16) + (u32::from(nolimit) << 8) + u32::from(positions);
        self.emplace_process(SpSummonStep {
            step: 0,
            zone,
            in_group: false,
            target,
        });
    }

    pub(crate) fn field_special_summon_complete(&mut self, reason_effect: Option<EffectId>, reason_player: u8) {
        let targets = std::mem::take(&mut self.core.special_summoning);
        self.emplace_process(SpSummon {
            step: 1,
            reason_player,
            zone: 0,
            reason_effect,
            targets,
        });
    }

    pub(crate) fn process_sp_summon_step(&mut self, arg: &mut SpSummonStep) -> bool {
        let target = arg.target;
        let param = self.cards[target].spsummon_param;
        let (playerid, nocheck, nolimit, positions) = (
            ((param >> 24) & 0xf) as u8,
            (param >> 16) & 0xff != 0,
            (param >> 8) & 0xff != 0,
            param as u8,
        );
        match arg.step {
            0 => {
                let card = &self.cards[target];
                let location = card.current.location;
                if !card.is_status(STATUS_PROC_COMPLETE)
                    && self.is_affected_by_effect(target, EFFECT_REVIVE_LIMIT).is_some()
                    && ((!nolimit && location & 0x38 != 0) || (!nocheck && !nolimit && location & 0x3 != 0))
                {
                    arg.step = 4;
                    return false;
                }
                let summon = self.cards[target].summon;
                if location == LOCATION_MZONE
                    || (u32::from(positions) & POS_FACEDOWN == 0 && self.check_unique_onfield(target, playerid, LOCATION_MZONE).is_some())
                    || !self.is_player_can_spsummon(
                        self.core.reason_effect.get(),
                        summon.summon_type & 0xff00_ffff,
                        positions,
                        summon.player,
                        playerid,
                        target,
                    )
                    || (!nocheck && self.cards[target].data.card_type & TYPE_MONSTER == 0)
                {
                    arg.step = 4;
                    return false;
                }
                if self
                    .get_useable_count(
                        Some(target),
                        playerid,
                        LOCATION_MZONE,
                        summon.player,
                        LOCATION_REASON_TOFIELD,
                        arg.zone,
                    )
                    .0
                    <= 0
                {
                    if location != LOCATION_GRAVE {
                        self.core.ss_tograve_set.insert(target);
                    }
                    arg.step = 4;
                    return false;
                }
                let (se, st) = (self.core.reason_effect.get(), summon.summon_type & 0xff00_ffff);
                if !nocheck && !self.is_spsummon_condition_met(target, se, summon.player, st, positions, playerid) {
                    arg.step = 4;
                    return false;
                }
                unported!(self, self.filter_effect(target, EFFECT_SPSUMMON_COST, true));
                false
            }
            1 => {
                if !arg.in_group {
                    self.core.special_summoning.insert(target);
                }
                self.enable_field_effect(target, false);
                // In a group the core narrows the zone by what the other summons leave free; for a card from outside
                // the Extra Deck that can only drop the Extra Monster Zones, which move_to_field leaves out of a full
                // zone mask anyway.
                let summon_player = self.cards[target].summon.player;
                let mut zone = arg.zone;
                if arg.in_group && self.is_flag(DUEL_EMZONE) && self.cards[target].current.location == LOCATION_EXTRA {
                    let (mut ct1, flag1) =
                        self.get_tofield_count(Some(target), playerid, LOCATION_MZONE, summon_player, LOCATION_REASON_TOFIELD, zone);
                    for card in self.core.spsummon_group {
                        let location = self.cards[card].current.location;
                        if location != LOCATION_MZONE && location != LOCATION_EXTRA {
                            ct1 -= 1;
                        }
                    }
                    if ct1 == 0 {
                        zone &= flag1;
                    }
                }
                let zone = zone as u8;
                self.field_move_to_field(
                    target,
                    summon_player,
                    playerid,
                    LOCATION_MZONE.into(),
                    positions,
                    false,
                    zone,
                    false,
                    LOCATION_REASON_SPSUMMON,
                );
                false
            }
            2 => {
                let card = &self.cards[target];
                let code = if card.current.position & POS_FACEDOWN == 0 {
                    card.data.code
                } else {
                    0
                };
                let location = self.cards.get_info_location(target);
                self.trace.message(MSG_SPSUMMONING).u32(code).loc(location);
                false
            }
            3 => {
                self.returns.set_i32(0, 1);
                let controler = self.cards[target].current.controler;
                self.set_control(target, controler, 0, 0);
                self.cards[target].set_status(STATUS_SPSUMMON_STEP, true);
                true
            }
            5 => {
                self.returns.set_i32(0, 0);
                self.restore_reason(target);
                if arg.in_group {
                    self.core.spsummon_group.remove(target);
                }
                true
            }
            _ => true,
        }
    }

    pub(crate) fn process_sp_summon(&mut self, arg: &mut SpSummon) -> bool {
        let turn_player = self.infos.turn_player;
        match arg.step {
            0 => {
                self.core.spsummon_group = arg.targets;
                let mut own = Cards16::default();
                let mut other = Cards16::default();
                for card in arg.targets {
                    if self.cards[card].summon.player == turn_player {
                        own.push(card);
                    } else {
                        other.push(card);
                    }
                }
                for (cards, player) in [(own, turn_player), (other, 1 - turn_player)] {
                    if cards.is_empty() {
                        continue;
                    }
                    let mut cards = cards;
                    cards.sort_by(|&c1, &c2| self.card_operation_sort(c1, c2));
                    self.core.hint_timing[player] |= TIMING_SPSUMMON;
                    for card in cards {
                        self.emplace_process(SpSummonStep {
                            step: 0,
                            zone: arg.zone,
                            in_group: true,
                            target: card,
                        });
                    }
                }
                false
            }
            1 => {
                let summoned: CardSet = arg
                    .targets
                    .iter()
                    .filter(|&card| self.cards[card].is_status(STATUS_SPSUMMON_STEP))
                    .collect();
                arg.targets = summoned;
                let tograve = self.core.ss_tograve_set;
                if !tograve.is_empty() {
                    self.send_to(
                        tograve,
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
            2 => {
                self.core.ss_tograve_set.clear();
                if arg.targets.is_empty() {
                    self.returns.set_i32(0, 0);
                    self.core.operated_set.clear();
                    return true;
                }
                // set_spsummon_counter and the once-per-turn summons come with the cards that limit special summons.
                for card in arg.targets {
                    self.cards[card].set_status(STATUS_SPSUMMON_STEP, false);
                    self.cards[card].set_status(STATUS_SPSUMMON_TURN, true);
                    if self.cards[card].is_position(POS_FACEUP) {
                        self.enable_field_effect(card, true);
                    }
                }
                self.adjust_instant();
                false
            }
            3 => {
                self.trace.message(MSG_SPSUMMONED);
                for card in arg.targets {
                    let current = self.cards[card].current;
                    if current.position & POS_FACEDOWN == 0 {
                        let summon_player = self.cards[card].summon.player;
                        self.raise_single_event(
                            card,
                            EVENT_SPSUMMON_SUCCESS,
                            current.reason_effect,
                            0,
                            current.reason_player,
                            summon_player,
                            0,
                        );
                    }
                    let summontype = self.cards[card].summon.summon_type & 0xff00_0000;
                    let materials = self.material_cards(card);
                    if summontype != 0 && !materials.is_empty() && !self.cards[card].is_status(STATUS_FUTURE_FUSION) {
                        let matreason = match summontype {
                            SUMMON_TYPE_FUSION => REASON_FUSION,
                            SUMMON_TYPE_RITUAL => REASON_RITUAL,
                            SUMMON_TYPE_XYZ => REASON_XYZ,
                            SUMMON_TYPE_LINK => REASON_LINK,
                            _ => 0,
                        };
                        let (summon_player, reason_effect, reason_player) =
                            (self.cards[card].summon.player, current.reason_effect, current.reason_player);
                        for material in materials {
                            self.raise_single_event(
                                material,
                                EVENT_BE_MATERIAL,
                                reason_effect,
                                matreason,
                                reason_player,
                                summon_player,
                                0,
                            );
                            if let Some(event) = self.core.single_event.last_mut() {
                                event.event_cards = Some(arg.targets);
                            }
                        }
                        self.field_raise_event(
                            Some(materials),
                            EVENT_BE_MATERIAL,
                            arg.reason_effect,
                            matreason,
                            arg.reason_player,
                            summon_player,
                            0,
                        );
                    }
                    self.cards[card].set_status(STATUS_FUTURE_FUSION, false);
                }
                self.process_single_event();
                self.process_instant_event();
                false
            }
            4 => {
                self.field_raise_event(
                    Some(arg.targets),
                    EVENT_SPSUMMON_SUCCESS,
                    arg.reason_effect,
                    0,
                    arg.reason_player,
                    PLAYER_NONE,
                    0,
                );
                self.process_instant_event();
                false
            }
            5 => {
                self.core.operated_set = arg.targets;
                self.returns.set_i32(0, arg.targets.len() as i32);
                true
            }
            _ => true,
        }
    }

    // SendTo step 4: shows the new deck top when cards leave the top of a reversed or face-up deck.
    fn check_deck_top_before_send(&mut self, cards: &[CardId]) {
        for p in 0..2 {
            let top = self.player[p].list_main.len() as i32 - 1;
            let mut s = top;
            for &card in cards {
                let current = self.cards[card].current;
                if current.location == LOCATION_DECK && current.controler == p && current.sequence as i32 == s {
                    s -= 1;
                }
            }
            if s == top || s <= 0 {
                continue;
            }
            let Some(&ptop) = self.player[p].list_main.get(s as usize) else {
                continue;
            };
            let (code, position) = (self.cards[ptop].data.code, self.cards[ptop].current.position);
            if self.core.deck_reversed || position == POS_FACEUP_DEFENSE {
                self.trace.message(MSG_DECK_TOP).u8(p).u32((top - s) as u32).u32(code).u32(position);
            }
        }
    }
}

// Normal Summons, Sets, Flip Summons and position changes: the parts of operations.cpp phase 1b reaches. Summon
// procedures, EFFECT_EXTRA_SUMMON_COUNT, summon costs, oaths and activity counters wait for effects (phase 2).
impl Duel {
    pub(crate) fn summon(
        &mut self,
        sumplayer: u8,
        target: CardId,
        summon_procedure_effect: Option<EffectId>,
        ignore_count: bool,
        min_tribute: u8,
        zone: u32,
    ) {
        let tributes = CardSet::default();
        self.emplace_process(SummonRule {
            step: 0,
            sumplayer,
            min_tribute,
            ignore_count,
            zone,
            target,
            summon_procedure_effect,
            extra_summon_effect: None,
            tributes,
            ordinary: false,
        });
    }

    pub(crate) fn mset(
        &mut self,
        setplayer: u8,
        target: CardId,
        summon_procedure_effect: Option<EffectId>,
        ignore_count: bool,
        min_tribute: u8,
        zone: u32,
    ) {
        let tributes = CardSet::default();
        self.emplace_process(MonsterSet {
            step: 0,
            setplayer,
            min_tribute,
            ignore_count,
            zone,
            target,
            summon_procedure_effect,
            extra_summon_effect: None,
            tributes,
            ordinary: false,
        });
    }

    pub(crate) fn field_release(&mut self, targets: CardSet, reason_effect: Option<EffectId>, reason: u32, reason_player: u8) {
        for card in targets.iter() {
            let target = &mut self.cards[card];
            target.temp.reason = target.current.reason;
            target.temp.reason_effect = target.current.reason_effect;
            target.temp.reason_player = target.current.reason_player;
            target.current.reason = reason;
            target.current.reason_effect = reason_effect;
            target.current.reason_player = reason_player;
            let owner = target.owner;
            target.sendto_param = SendtoParam {
                playerid: owner,
                position: POS_FACEUP as u8,
                location: LOCATION_GRAVE,
                sequence: 0,
            };
        }
        self.emplace_process(Release {
            step: 0,
            reason_player,
            reason,
            targets,
            reason_effect,
        });
    }

    // Takes the core's defaults for `ret` and `confirm`.
    pub(crate) fn field_move_to_field(
        &mut self,
        target: CardId,
        move_player: u8,
        playerid: u8,
        destination: u16,
        positions: u8,
        enable: bool,
        zone: u8,
        rule: bool,
        location_reason: u32,
    ) {
        let unit = MoveToField {
            step: 0,
            enable,
            pzone: false,
            zone,
            rule,
            location_reason,
            confirm: true,
            ret: 0,
            target,
        };
        self.move_to_field_unit(move_player, playerid, destination, positions, unit);
    }

    // move_to_field with every parameter in `unit`, whose zone and Pendulum Zone flag the destination may override.
    pub(crate) fn move_to_field_unit(&mut self, move_player: u8, playerid: u8, destination: u16, positions: u8, unit: MoveToField) {
        let target = unit.target;
        let to_field = u16::from(LOCATION_ONFIELD) | LOCATION_MMZONE | LOCATION_EMZONE | LOCATION_STZONE | LOCATION_PZONE | LOCATION_FZONE;
        let current = self.cards[target].current;
        if destination & to_field == 0
            || positions == 0
            || (destination & LOCATION_PZONE != 0 && current.is_location(LOCATION_PZONE) && playerid == current.controler)
            || (destination == u16::from(current.location) && playerid == current.controler)
        {
            return;
        }
        let (destination, zone, pzone) = match destination {
            LOCATION_PZONE => (LOCATION_SZONE, unit.zone, true),
            LOCATION_FZONE => (LOCATION_SZONE, 1 << 5, false),
            LOCATION_MMZONE => (LOCATION_MZONE, 0x1f, false),
            LOCATION_STZONE => (LOCATION_SZONE, 0x1f, false),
            LOCATION_EMZONE => (LOCATION_MZONE, 0x60, false),
            _ => (destination as u8, unit.zone, false),
        };
        self.cards[target].to_field_param =
            (u32::from(move_player) << 24) + (u32::from(playerid) << 16) + (u32::from(destination) << 8) + u32::from(positions);
        self.emplace_process(MoveToField { zone, pzone, ..unit });
    }

    pub(crate) fn change_position(
        &mut self,
        target: CardId,
        reason_effect: Option<EffectId>,
        reason_player: u8,
        npos: u32,
        flag: u32,
        enable: bool,
    ) {
        self.cards[target].position_param = npos | flag;
        let (targets, to_grave_set) = (target.into(), CardSet::default());
        self.emplace_process(ChangePos {
            step: 0,
            reason_player,
            enable,
            oppo_selection: false,
            reason_effect,
            targets,
            to_grave_set,
        });
    }

    pub(crate) fn select_tribute_cards(
        &mut self,
        target: Option<CardId>,
        playerid: u8,
        cancelable: bool,
        min: i32,
        max: i32,
        toplayer: u8,
        zone: u32,
    ) {
        let (min, max, zone, must_choose_one) = (min as u16, max as u16, zone as u8, CardSet::default());
        self.emplace_process(SelectTribute {
            step: 0,
            min,
            max,
            playerid,
            cancelable,
            toplayer,
            zone,
            target,
            extra_release_effect: None,
            must_choose_one,
        });
    }

    // Step 2 of SummonRule and MonsterSet without a procedure: the tribute selection, if the card needs one.
    fn select_summon_tributes(&mut self, target: CardId, player: u8, required: i32, min_tribute: u8, advance: bool, zone: u32) {
        self.core.select_cards.clear();
        let (min, max) = ((required & 0xffff).max(i32::from(min_tribute)), required >> 16);
        if max == 0 || !advance {
            self.return_cards.clear();
            return;
        }
        let (mut release_cards, mut release_cards_ex) = (CardSet::default(), CardSet::default());
        let rcount = self.get_summon_release_list(target, &mut release_cards, &mut release_cards_ex, None, false, 0xff00ff);
        self.core.release_cards = release_cards;
        self.core.release_cards_ex = release_cards_ex;
        self.core.release_cards_ex_oneof.clear();
        if rcount == 0 {
            self.return_cards.clear();
            return;
        }
        // Without EFFECT_DECREASE_TRIBUTE min is 0 only when max is, so the core's SelectYesNo for optional tributes
        // (hint 90, then step 3) cannot come up before phase 2.
        let fcount = self.get_mzone_limit(player, player, LOCATION_REASON_TOFIELD);
        let cancelable = self.core.summon_cancelable;
        self.select_tribute_cards(Some(target), player, cancelable, min.max(1 - fcount), max, player, zone);
    }

    fn release_tributes(&mut self, target: CardId, tributes: CardSet, player: u8) {
        if tributes.is_empty() {
            return;
        }
        self.set_material(target, tributes);
        self.field_release(tributes, None, REASON_SUMMON | REASON_MATERIAL, player);
        self.cards[target].summon.summon_type |= SUMMON_TYPE_ADVANCE;
        self.adjust_all();
    }

    fn raise_material_events(&mut self, materials: CardSet, event: u32, reason_effect: Option<EffectId>, player: u8) {
        if materials.is_empty() {
            return;
        }
        for material in materials.iter() {
            self.raise_single_event(material, event, reason_effect, REASON_SUMMON, player, player, 0);
        }
        self.field_raise_event(Some(materials), event, reason_effect, REASON_SUMMON, player, player, 0);
    }

    fn add_summon_count(&mut self, extra_summon_effect: Option<EffectId>, player: u8) {
        unported!(self, extra_summon_effect);
        self.core.summon_count[player] += 1;
    }

    // Step 3 (optional tributes) needs EFFECT_DECREASE_TRIBUTE.
    pub(crate) fn process_summon_rule(&mut self, arg: &mut SummonRule) -> bool {
        let (sumplayer, target, procedure) = (arg.sumplayer, arg.target, arg.summon_procedure_effect);
        let procedure_args = Procedure {
            card: target,
            min: u16::from(arg.min_tribute),
            zone: arg.zone,
            releasable: 0xff00ff,
            ..Procedure::default()
        };
        match arg.step {
            0 => {
                if !self.is_summonable_card(target)
                    || self.check_unique_onfield(target, sumplayer, LOCATION_MZONE).is_some()
                    || self.is_affected_by_effect(target, EFFECT_CANNOT_SUMMON).is_some()
                {
                    return true;
                }
                if self.cards[target].current.location == LOCATION_MZONE {
                    if self.cards[target].is_position(POS_FACEDOWN)
                        || (!arg.ignore_count
                            && (self.core.extra_summon[sumplayer] != 0
                                || self.is_affected_by_effect(target, EFFECT_EXTRA_SUMMON_COUNT).is_none())
                            && self.core.summon_count[sumplayer] >= self.get_summon_count_limit(sumplayer))
                        || self.is_affected_by_effect(target, EFFECT_GEMINI_SUMMONABLE).is_none()
                        || self.is_affected_by_effect(target, EFFECT_GEMINI_STATUS).is_some()
                        || !self.is_player_can_summon(SUMMON_TYPE_GEMINI, sumplayer, target, sumplayer)
                    {
                        return true;
                    }
                } else {
                    let (result, procedures) = self.filter_summon_procedure(target, sumplayer, arg.ignore_count, arg.min_tribute, arg.zone);
                    // Duel.Summon with a given procedure (c80921533) comes with the cards that pass one.
                    unported!(self, procedure);
                    if result == -2 {
                        return true;
                    }
                    arg.ordinary = result > 0;
                    let descriptions = procedures.iter().map(|&effect| self.effects[effect].description);
                    self.core.select_options = arg.ordinary.then_some(1).into_iter().chain(descriptions).collect();
                    self.core.select_effects = procedures.iter().copied().collect();
                    match self.core.select_options.len() {
                        0 => return true,
                        1 => self.returns.set_i32(0, 0),
                        _ => self.emplace_process(SelectOption {
                            playerid: sumplayer,
                            ..SelectOption::default()
                        }),
                    }
                }
                if self.core.summon_depth != 0 {
                    self.core.summon_cancelable = false;
                }
                self.core.summon_depth += 1;
                self.set_material_cards(target, CardSet::default());
                false
            }
            1 => {
                let extra = self.filter_effect(target, EFFECT_EXTRA_SUMMON_COUNT, true);
                if !arg.ignore_count && self.core.extra_summon[sumplayer] == 0 {
                    unported!(self, extra);
                }
                if self.cards[target].current.location == LOCATION_MZONE {
                    arg.step = 4;
                    return false;
                }
                if arg.summon_procedure_effect.is_none() {
                    let index = usize::try_from(self.returns.at_i32(0)).unwrap_or(0);
                    let index = if arg.ordinary { index.checked_sub(1) } else { Some(index) };
                    arg.summon_procedure_effect = index.and_then(|index| self.core.select_effects.get(index)).copied();
                }
                self.returns.set_i32(0, 0);
                false
            }
            2 => {
                if procedure.is_some() {
                    arg.step = 3;
                    return false;
                }
                let required = self.get_summon_tribute_count(target);
                let advance = self.is_player_can_summon(SUMMON_TYPE_ADVANCE, sumplayer, target, sumplayer);
                self.select_summon_tributes(target, sumplayer, required, arg.min_tribute, advance, arg.zone);
                arg.step = 4;
                false
            }
            4 => {
                self.returns.set_i32(0, 1);
                if let Some(procedure) = procedure
                    && matches!(self.effects[procedure].target, Target::Action(_))
                {
                    let nil_event = self.nil_event;
                    self.execute_procedure(ExecuteKind::Target, procedure, sumplayer, nil_event, Some(procedure_args));
                }
                false
            }
            5 => {
                if self.cards[target].current.location == LOCATION_MZONE {
                    arg.step = 9;
                } else if procedure.is_some() {
                    if self.returns.at_i32(0) == 0 {
                        self.core.summon_depth -= 1;
                        return true;
                    }
                    arg.step = 6;
                } else {
                    if self.return_cards.canceled {
                        self.core.summon_depth -= 1;
                        return true;
                    }
                    arg.tributes = self.return_cards.list.iter().copied().collect();
                }
                unported!(self, self.filter_effect(target, EFFECT_SUMMON_COST, true));
                false
            }
            6 => {
                let required = match self.get_level(target) {
                    ..5 => 0,
                    5..7 => 1,
                    _ => 2,
                };
                if required > arg.tributes.len() {
                    unported!(self, self.filter_effect(target, EFFECT_DECREASE_TRIBUTE, true));
                }
                let summon = &mut self.cards[target].summon;
                (summon.location, summon.summon_type, summon.pzone) = (LOCATION_HAND, SUMMON_TYPE_NORMAL, false);
                self.release_tributes(target, arg.tributes, sumplayer);
                self.cards[target].current.reason_effect = None;
                self.cards[target].current.reason_player = sumplayer;
                arg.step = 7;
                false
            }
            7 => {
                let Some(procedure) = procedure else { return false };
                let summon_type = (self.get_value(procedure, target) as u32 & 0xfff_ffff) | SUMMON_TYPE_NORMAL;
                let card = &mut self.cards[target];
                (card.summon.location, card.summon.pzone, card.summon.summon_type) = (LOCATION_HAND, false, summon_type);
                card.current.reason_effect = Some(procedure);
                card.current.reason_player = sumplayer;
                self.returns.set_i32(0, 1);
                if self.effects[procedure].operation.is_some() {
                    let nil_event = self.nil_event;
                    self.execute_procedure(ExecuteKind::Operation, procedure, sumplayer, nil_event, Some(procedure_args));
                }
                self.dec_count(procedure, sumplayer);
                false
            }
            8 => {
                self.core.summon_depth -= 1;
                if self.core.summon_depth != 0 {
                    return true;
                }
                self.field_break_effect(true);
                if !arg.ignore_count {
                    self.add_summon_count(arg.extra_summon_effect, sumplayer);
                }
                false
            }
            9 => {
                let faceup_defense = self.is_flag(DUEL_NORMAL_SUMMON_FACEUP_DEF)
                    || self
                        .is_player_affected_by_effect(sumplayer, EFFECT_NORMAL_SUMMON_FACEUP_DEFENSE)
                        .is_some()
                    || self.is_player_affected_by_effect(sumplayer, EFFECT_DEVINE_LIGHT).is_some();
                let mut positions = (if faceup_defense { POS_FACEUP } else { POS_FACEUP_ATTACK }) as u8;
                let mut targetplayer = sumplayer;
                if let Some(procedure) = procedure
                    && self.effects[procedure].is_flag(EFFECT_FLAG_SPSUM_PARAM)
                {
                    positions = self.effects[procedure].s_range as u8 & POS_FACEUP as u8;
                    if self.effects[procedure].o_range != 0 {
                        targetplayer = 1 - sumplayer;
                    }
                }
                unported!(
                    self,
                    self.filter_player_effect(sumplayer, EFFECT_FORCE_NORMAL_SUMMON_POSITION, true)
                );
                self.enable_field_effect(target, false);
                let (destination, zone) = (LOCATION_MZONE.into(), arg.zone as u8);
                self.field_move_to_field(
                    target,
                    sumplayer,
                    targetplayer,
                    destination,
                    positions,
                    false,
                    zone,
                    false,
                    LOCATION_REASON_SUMMON,
                );
                arg.step = 11;
                false
            }
            10 => {
                self.core.summon_depth -= 1;
                if self.core.summon_depth != 0 {
                    return true;
                }
                self.enable_field_effect(target, false);
                let card = &mut self.cards[target];
                card.summon.location = LOCATION_MZONE;
                card.summon.pzone = false;
                card.summon.sequence = card.current.sequence;
                card.summon.summon_type |= SUMMON_TYPE_NORMAL;
                card.current.reason_effect = None;
                card.current.reason_player = sumplayer;
                self.add_status_effect(target, EFFECT_GEMINI_STATUS, 64, RESET_EVENT + 0x1fe_0000);
                false
            }
            11 => {
                if !arg.ignore_count {
                    self.add_summon_count(arg.extra_summon_effect, sumplayer);
                }
                false
            }
            12 => {
                let controler = self.cards[target].current.controler;
                self.set_control(target, controler, 0, 0);
                self.core.phase_action = true;
                self.cards[target].current.reason = REASON_SUMMON;
                self.cards[target].summon.player = sumplayer;
                let (code, location) = (self.cards[target].data.code, self.cards.get_info_location(target));
                self.trace.message(MSG_SUMMONING).u32(code).loc(location);
                if self.is_flag(DUEL_CANNOT_SUMMON_OATH_OLD) {
                    self.core.summon_state_count[sumplayer] += 1;
                    self.core.normalsummon_state_count[sumplayer] += 1;
                }
                self.raise_material_events(arg.tributes, EVENT_BE_PRE_MATERIAL, procedure, sumplayer);
                self.process_single_event();
                self.process_instant_event();
                false
            }
            13 => {
                if !self.core.current_chain.is_empty() || self.is_affected_by_effect(target, EFFECT_CANNOT_DISABLE_SUMMON).is_some() {
                    arg.step = 15;
                }
                false
            }
            14 => {
                self.cards[target].set_status(STATUS_SUMMONING, true);
                self.cards[target].set_status(STATUS_SUMMON_DISABLED, false);
                self.field_raise_event(Some(target.into()), EVENT_SUMMON, procedure, 0, sumplayer, sumplayer, 0);
                self.process_instant_event();
                self.emplace_process(PointEvent {
                    skip_trigger: true,
                    skip_freechain: true,
                    skip_new: true,
                    ..PointEvent::default()
                });
                false
            }
            15 => {
                if self.cards[target].is_status(STATUS_SUMMONING) {
                    arg.step = 15;
                    return false;
                }
                if self.cards[target].current.location == LOCATION_MZONE {
                    self.send_to(target.into(), None, REASON_RULE, sumplayer, sumplayer, LOCATION_GRAVE, 0, 0, false);
                }
                self.adjust_instant();
                self.emplace_process(PointEvent::default());
                true
            }
            16 => {
                self.cards[target].set_status(STATUS_SUMMONING, false);
                self.cards[target].set_status(STATUS_SUMMON_TURN, true);
                self.enable_field_effect(target, true);
                if self.cards[target].is_status(STATUS_DISABLED) {
                    self.reset(target, RESET_DISABLE, RESET_EVENT);
                }
                false
            }
            17 => {
                self.trace.message(MSG_SUMMONED);
                self.adjust_instant();
                self.raise_material_events(arg.tributes, EVENT_BE_MATERIAL, procedure, sumplayer);
                self.process_single_event();
                self.process_instant_event();
                false
            }
            18 => {
                if !self.is_flag(DUEL_CANNOT_SUMMON_OATH_OLD) {
                    self.core.summon_state_count[sumplayer] += 1;
                    self.core.normalsummon_state_count[sumplayer] += 1;
                }
                self.raise_single_event(target, EVENT_SUMMON_SUCCESS, procedure, 0, sumplayer, sumplayer, 0);
                self.process_single_event();
                self.field_raise_event(Some(target.into()), EVENT_SUMMON_SUCCESS, procedure, 0, sumplayer, sumplayer, 0);
                self.process_instant_event();
                if self.core.current_chain.is_empty() {
                    self.adjust_all();
                    self.core.hint_timing[sumplayer] |= TIMING_SUMMON;
                    self.emplace_process(PointEvent::default());
                }
                true
            }
            _ => true,
        }
    }

    pub(crate) fn process_flip_summon(&mut self, arg: &mut FlipSummon) -> bool {
        let (sumplayer, target) = (arg.sumplayer, arg.target);
        match arg.step {
            0 => {
                if self.cards[target].current.location != LOCATION_MZONE
                    || !self.cards[target].is_position(POS_FACEDOWN)
                    || self.check_unique_onfield(target, sumplayer, LOCATION_MZONE).is_some()
                {
                    return true;
                }
                unported!(self, self.filter_effect(target, EFFECT_FLIPSUMMON_COST, true));
                false
            }
            1 => {
                let field_id = self.next_field_id();
                let card = &mut self.cards[target];
                card.previous.position = card.current.position;
                card.current.position = POS_FACEUP_ATTACK;
                card.summon.player = sumplayer;
                card.fieldid = field_id;
                self.core.phase_action = true;
                if self.is_flag(DUEL_CANNOT_SUMMON_OATH_OLD) {
                    self.core.flipsummon_state_count[sumplayer] += 1;
                }
                let (code, location) = (self.cards[target].data.code, self.cards.get_info_location(target));
                self.trace.message(MSG_FLIPSUMMONING).u32(code).loc(location);
                if self.is_affected_by_effect(target, EFFECT_CANNOT_DISABLE_FLIP_SUMMON).is_some() {
                    arg.step = 2;
                    return false;
                }
                self.cards[target].set_status(STATUS_SUMMONING, true);
                self.cards[target].set_status(STATUS_SUMMON_DISABLED, false);
                self.field_raise_event(Some(target.into()), EVENT_FLIP_SUMMON, None, 0, sumplayer, sumplayer, 0);
                self.process_instant_event();
                self.emplace_process(PointEvent {
                    skip_trigger: true,
                    skip_freechain: true,
                    skip_new: true,
                    ..PointEvent::default()
                });
                false
            }
            2 => {
                if self.cards[target].is_status(STATUS_SUMMONING) {
                    return false;
                }
                if self.cards[target].current.location == LOCATION_MZONE {
                    self.send_to(target.into(), None, REASON_RULE, sumplayer, sumplayer, LOCATION_GRAVE, 0, 0, false);
                }
                self.emplace_process(PointEvent::default());
                true
            }
            3 => {
                self.cards[target].set_status(STATUS_SUMMONING, false);
                self.enable_field_effect(target, true);
                if self.cards[target].is_status(STATUS_DISABLED) {
                    self.reset(target, RESET_DISABLE, RESET_EVENT);
                }
                self.cards[target].set_status(STATUS_FLIP_SUMMON_TURN, true);
                false
            }
            4 => {
                self.trace.message(MSG_FLIPSUMMONED);
                if !self.is_flag(DUEL_CANNOT_SUMMON_OATH_OLD) {
                    self.core.flipsummon_state_count[sumplayer] += 1;
                }
                self.adjust_instant();
                let events = [EVENT_FLIP, EVENT_FLIP_SUMMON_SUCCESS, EVENT_CHANGE_POS];
                for event in events {
                    self.raise_single_event(target, event, None, 0, sumplayer, sumplayer, 0);
                }
                self.process_single_event();
                for event in events {
                    self.field_raise_event(Some(target.into()), event, None, 0, sumplayer, sumplayer, 0);
                }
                self.process_instant_event();
                self.adjust_all();
                if self.core.current_chain.is_empty() {
                    self.core.hint_timing[sumplayer] |= TIMING_FLIPSUMMON;
                    self.emplace_process(PointEvent::default());
                }
                true
            }
            _ => true,
        }
    }

    // Step 3 (optional tributes) needs EFFECT_DECREASE_TRIBUTE_SET.
    pub(crate) fn process_monster_set(&mut self, arg: &mut MonsterSet) -> bool {
        let (setplayer, target, procedure) = (arg.setplayer, arg.target, arg.summon_procedure_effect);
        let procedure_args = Procedure {
            card: target,
            min: u16::from(arg.min_tribute),
            zone: arg.zone,
            releasable: 0xff00ff,
            ..Procedure::default()
        };
        match arg.step {
            0 => {
                let card = &self.cards[target];
                if self.is_affected_by_effect(target, EFFECT_UNSUMMONABLE_CARD).is_some()
                    || card.current.location != LOCATION_HAND
                    || card.data.card_type & TYPE_MONSTER == 0
                    || self.is_affected_by_effect(target, EFFECT_CANNOT_MSET).is_some()
                {
                    return true;
                }
                let (result, procedures) = self.filter_set_procedure(target, setplayer, arg.ignore_count, arg.min_tribute, arg.zone);
                // Duel.MSet with a given procedure comes with the cards that pass one.
                unported!(self, procedure);
                if result == -2 {
                    return true;
                }
                arg.ordinary = result > 0;
                let descriptions = procedures.iter().map(|&effect| self.effects[effect].description);
                self.core.select_options = arg.ordinary.then_some(1).into_iter().chain(descriptions).collect();
                self.core.select_effects = procedures.iter().copied().collect();
                match self.core.select_options.len() {
                    0 => return true,
                    1 => self.returns.set_i32(0, 0),
                    _ => self.emplace_process(SelectOption {
                        playerid: setplayer,
                        ..SelectOption::default()
                    }),
                }
                self.set_material_cards(target, CardSet::default());
                false
            }
            1 => {
                let extra = self.filter_effect(target, EFFECT_EXTRA_SET_COUNT, true);
                if arg.summon_procedure_effect.is_none() {
                    let index = usize::try_from(self.returns.at_i32(0)).unwrap_or(0);
                    let index = if arg.ordinary { index.checked_sub(1) } else { Some(index) };
                    arg.summon_procedure_effect = index.and_then(|index| self.core.select_effects.get(index)).copied();
                }
                if !arg.ignore_count && self.core.extra_summon[setplayer] == 0 {
                    unported!(self, extra);
                }
                self.returns.set_i32(0, 0);
                false
            }
            2 => {
                if procedure.is_some() {
                    arg.step = 3;
                    return false;
                }
                let required = self.get_set_tribute_count(target);
                let advance = self.is_player_can_mset(SUMMON_TYPE_ADVANCE, setplayer, target, setplayer);
                self.select_summon_tributes(target, setplayer, required, arg.min_tribute, advance, arg.zone);
                arg.step = 3;
                false
            }
            4 => {
                if procedure.is_some() {
                    arg.step = 5;
                } else {
                    if self.return_cards.canceled {
                        return true;
                    }
                    if !self.return_cards.list.is_empty() {
                        arg.tributes = self.return_cards.list.iter().copied().collect();
                    }
                }
                unported!(self, self.filter_effect(target, EFFECT_MSET_COST, true));
                false
            }
            5 => {
                let summon = &mut self.cards[target].summon;
                (summon.location, summon.pzone, summon.summon_type) = (LOCATION_HAND, false, SUMMON_TYPE_NORMAL);
                self.release_tributes(target, arg.tributes, setplayer);
                self.cards[target].summon.player = setplayer;
                self.cards[target].current.reason_effect = None;
                self.cards[target].current.reason_player = setplayer;
                arg.step = 7;
                false
            }
            6 => {
                self.returns.set_i32(0, 1);
                if let Some(procedure) = procedure
                    && matches!(self.effects[procedure].target, Target::Action(_))
                {
                    let nil_event = self.nil_event;
                    self.execute_procedure(ExecuteKind::Target, procedure, setplayer, nil_event, Some(procedure_args));
                }
                false
            }
            7 => {
                let Some(procedure) = procedure else { return false };
                if self.returns.at_i32(0) == 0 {
                    return true;
                }
                let summon_type = (self.get_value(procedure, target) as u32 & 0xfff_ffff) | SUMMON_TYPE_NORMAL;
                let card = &mut self.cards[target];
                (card.summon.summon_type, card.summon.location, card.summon.pzone) = (summon_type, LOCATION_HAND, false);
                card.summon.player = setplayer;
                card.current.reason_effect = Some(procedure);
                card.current.reason_player = setplayer;
                if self.effects[procedure].operation.is_some() {
                    let nil_event = self.nil_event;
                    self.execute_procedure(ExecuteKind::Operation, procedure, setplayer, nil_event, Some(procedure_args));
                }
                self.dec_count(procedure, setplayer);
                false
            }
            8 => {
                self.field_break_effect(true);
                if !arg.ignore_count {
                    self.add_summon_count(arg.extra_summon_effect, setplayer);
                }
                false
            }
            9 => {
                let (mut positions, mut targetplayer) = (POS_FACEDOWN_DEFENSE as u8, setplayer);
                if let Some(procedure) = procedure
                    && self.effects[procedure].is_flag(EFFECT_FLAG_SPSUM_PARAM)
                {
                    positions = self.effects[procedure].s_range as u8 & POS_FACEDOWN as u8;
                    if self.effects[procedure].o_range != 0 {
                        targetplayer = 1 - setplayer;
                    }
                }
                self.enable_field_effect(target, false);
                let (destination, zone) = (LOCATION_MZONE.into(), arg.zone as u8);
                self.field_move_to_field(
                    target,
                    setplayer,
                    targetplayer,
                    destination,
                    positions,
                    false,
                    zone,
                    false,
                    LOCATION_REASON_SET,
                );
                false
            }
            10 => {
                let controler = self.cards[target].current.controler;
                self.set_control(target, controler, 0, 0);
                self.core.phase_action = true;
                self.core.normalsummon_state_count[setplayer] += 1;
                self.cards[target].set_status(STATUS_SUMMON_TURN, true);
                let (code, location) = (self.cards[target].data.code, self.cards.get_info_location(target));
                self.trace.message(MSG_SET).u32(code).loc(location);
                self.adjust_instant();
                self.field_raise_event(Some(target.into()), EVENT_MSET, procedure, 0, setplayer, setplayer, 0);
                self.process_instant_event();
                if self.core.current_chain.is_empty() {
                    self.adjust_all();
                    self.core.hint_timing[setplayer] |= TIMING_MSET;
                    self.emplace_process(PointEvent::default());
                }
                true
            }
            _ => true,
        }
    }

    // The core runs ReleaseReplace per card after step 1; here its checks run inline, as SendTo's do.
    pub(crate) fn process_release(&mut self, arg: &mut Release) -> bool {
        let (reason_effect, reason, reason_player) = (arg.reason_effect, arg.reason, arg.reason_player);
        match arg.step {
            0 => {
                let targets = arg.targets;
                for card in targets.iter() {
                    let target = &self.cards[card];
                    let current = target.current;
                    let rejected = target.get_status(STATUS_SUMMONING | STATUS_SPSUMMON_STEP)
                        || (reason & REASON_SUMMON != 0
                            && current
                                .reason_card
                                .is_none_or(|summoned| !self.is_releasable_by_summon(card, reason_player, summoned)))
                        || (current.reason & (REASON_RULE | REASON_SUMMON | REASON_COST) == 0
                            && (!self.is_affect_by_effect(card, current.reason_effect)
                                || !self.is_releasable_by_nonsummon(card, reason_player, reason)));
                    if rejected {
                        self.restore_reason(card);
                        arg.targets.remove(card);
                    }
                }
                if reason & REASON_RULE == 0 {
                    // operation_replace(EFFECT_RELEASE_REPLACE, 5, targets)
                    unported!(self, self.field_effect_range(Container::Continuous, EFFECT_RELEASE_REPLACE));
                }
                false
            }
            1 => {
                let targets = arg.targets;
                for card in targets.iter() {
                    if self.cards[card].current.location & (LOCATION_GRAVE | LOCATION_REMOVED) != 0 {
                        self.restore_reason(card);
                        arg.targets.remove(card);
                    } else if self.cards[card].current.reason & REASON_RULE == 0 {
                        self.returns.set_i32(0, 0);
                        unported!(
                            self,
                            self.field_effects
                                .registry
                                .equal_range(Container::Single, Some(card), EFFECT_RELEASE_REPLACE)
                        );
                    }
                }
                false
            }
            2 => {
                if arg.targets.is_empty() {
                    self.returns.set_i32(0, 0);
                    self.core.operated_set.clear();
                    return true;
                }
                let mut cv: Cards16 = arg.targets.iter().collect();
                cv.sort_by(|&c1, &c2| self.card_operation_sort(c1, c2));
                for card in cv {
                    if self.cards[card].current.location & (LOCATION_GRAVE | LOCATION_REMOVED) != 0 {
                        self.restore_reason(card);
                        arg.targets.remove(card);
                    } else {
                        self.cards[card].current.reason |= REASON_RELEASE;
                    }
                }
                self.adjust_instant();
                false
            }
            3 => {
                // operation_replace(EFFECT_SEND_REPLACE, 5, targets)
                unported!(self, self.field_effect_range(Container::Continuous, EFFECT_SEND_REPLACE));
                let (targets, reason) = (arg.targets, reason | REASON_RELEASE);
                self.emplace_process(SendTo {
                    step: 1,
                    targets,
                    reason_effect,
                    reason,
                    reason_player,
                    extra_args: None,
                });
                false
            }
            4 => {
                self.core.operated_set = arg.targets;
                self.returns.set_i32(0, arg.targets.len() as i32);
                true
            }
            _ => true,
        }
    }

    // The field spell and Pendulum Zone branches of step 0 place spells (phase 2).
    pub(crate) fn process_move_to_field(&mut self, arg: &mut MoveToField) -> bool {
        let target = arg.target;
        let param = self.cards[target].to_field_param;
        let (move_player, playerid, location, positions) = ((param >> 24) as u8, (param >> 16) as u8, (param >> 8) as u8, param as u8);
        let reason = arg.location_reason;
        match arg.step {
            0 => {
                self.returns.set_i32(0, 0);
                if arg.ret == 1 {
                    let current = self.cards[target].current;
                    let owner = |effect: Option<EffectId>| effect.map(|effect| self.effects[effect].owner);
                    if current.reason & REASON_TEMPORARY == 0 || owner(current.reason_effect) != owner(self.core.reason_effect.get()) {
                        return true;
                    }
                }
                let data_type = self.cards[target].data.card_type;
                unported!(
                    self,
                    (location == LOCATION_SZONE && arg.zone == 1 << 5 && data_type & TYPE_FIELD != 0).then_some(target)
                );
                if arg.pzone && location == LOCATION_SZONE && data_type & TYPE_PENDULUM != 0 && self.is_flag(DUEL_PZONE) {
                    let mut flag = 0u32;
                    for index in 0..2 {
                        if self.is_location_useable(playerid, LOCATION_PZONE, index) && arg.zone & (1 << index) != 0 {
                            flag |= 1 << (self.get_pzone_index(index) + 8);
                        }
                    }
                    if flag == 0 {
                        return true;
                    }
                    if move_player != playerid {
                        flag <<= 16;
                    }
                    let code = self.cards[target].data.code;
                    self.trace.message(MSG_HINT).u8(HINT_SELECTMSG).u8(move_player).u64(u64::from(code));
                    self.emplace_process(SelectPlace {
                        playerid: move_player,
                        count: 1,
                        flag: !flag,
                        ..SelectPlace::default()
                    });
                    return false;
                }
                let real_reason = if reason != LOCATION_REASON_NONE {
                    if reason != LOCATION_REASON_SUMMON && reason != LOCATION_REASON_SPSUMMON && reason != LOCATION_REASON_SET {
                        reason
                    } else {
                        LOCATION_REASON_TOFIELD
                    }
                } else if self.cards[target].current.location == LOCATION_MZONE {
                    LOCATION_REASON_CONTROL
                } else {
                    LOCATION_REASON_TOFIELD
                };
                let zone = arg.zone;
                let (mut ct, mut flag) =
                    self.get_useable_count(Some(target), playerid, location, move_player, real_reason, u32::from(zone));
                if location == LOCATION_MZONE && zone & 0x60 != 0 && zone != 0xff && !arg.rule {
                    for sequence in [5, 6] {
                        if zone & (1 << sequence) != 0 && self.is_location_useable(playerid, location.into(), sequence) {
                            flag &= !(1 << sequence);
                            ct += 1;
                        }
                    }
                }
                if location == LOCATION_SZONE {
                    flag |= !u32::from(zone);
                }
                if arg.ret == 1
                    && (ct <= 0
                        || self.cards[target].is_status(STATUS_FORBIDDEN)
                        || (u32::from(positions) & POS_FACEDOWN == 0 && self.check_unique_onfield(target, playerid, location).is_some()))
                {
                    arg.step = 3;
                    let (reason_effect, reason_player) = (self.core.reason_effect.get(), self.core.reason_player.get());
                    self.send_to(
                        target.into(),
                        reason_effect,
                        REASON_RULE,
                        reason_player,
                        PLAYER_NONE,
                        LOCATION_GRAVE,
                        0,
                        0,
                        false,
                    );
                    return false;
                }
                if ct <= 0 || !flag == 0 {
                    return true;
                }
                if !arg.confirm
                    && zone & zone.wrapping_sub(1) == 0
                    && let Some(sequence) = (0..8).find(|sequence| (1 << sequence) & zone != 0)
                {
                    self.returns.set_u8(2, sequence);
                    return false;
                }
                if reason == LOCATION_REASON_SUMMON || reason == LOCATION_REASON_SPSUMMON || reason == LOCATION_REASON_SET {
                    unported!(
                        self,
                        self.filter_player_effect(move_player, EFFECT_OPPO_CHOOSES_SPSUMMON_ZONE, true)
                    );
                }
                let flag = match (move_player == playerid, location == LOCATION_SZONE) {
                    (true, true) => ((flag & 0xff) << 8) | 0xffff_00ff,
                    (true, false) => (flag & 0xff) | 0xffff_ff00,
                    (false, true) => ((flag & 0xff) << 24) | 0xff_ffff,
                    (false, false) => ((flag & 0xff) << 16) | 0xff00_ffff,
                } | 0xe080_e080;
                self.trace
                    .message(MSG_HINT)
                    .u8(HINT_SELECTMSG)
                    .u8(move_player)
                    .u64(u64::from(self.cards[target].data.code));
                self.emplace_process(SelectPlace {
                    playerid: move_player,
                    count: 1,
                    flag,
                    ..SelectPlace::default()
                });
                false
            }
            1 => {
                let sequence = u32::from(self.returns.at_u8(2));
                let current = self.cards[target].current;
                if arg.ret != 1 && location != current.location {
                    let mut resetflag = 0;
                    if location & LOCATION_ONFIELD != 0 {
                        resetflag |= RESET_TOFIELD;
                    }
                    if current.location & LOCATION_ONFIELD != 0 {
                        resetflag |= RESET_LEAVE;
                    }
                    unported!(self, self.is_affected_by_effect(target, EFFECT_PRE_MONSTER));
                    if location & LOCATION_ONFIELD != 0 && current.location & LOCATION_ONFIELD != 0 {
                        resetflag |= RESET_MSCHANGE;
                    }
                    self.reset(target, resetflag, RESET_EVENT);
                    self.clear_card_target(target);
                }
                if current.location & LOCATION_ONFIELD == 0 {
                    self.clear_relate_effect(target);
                }
                if arg.ret == 1 {
                    self.cards[target].current.reason &= !REASON_TEMPORARY;
                }
                if (arg.ret == 0 && location != current.location) || arg.ret == 1 {
                    let turn_status =
                        STATUS_SUMMON_TURN | STATUS_FLIP_SUMMON_TURN | STATUS_SPSUMMON_TURN | STATUS_SET_TURN | STATUS_FORM_CHANGED;
                    self.cards[target].set_status(turn_status, false);
                }
                self.cards[target].temp.sequence = sequence;
                let card_type = self.cards[target].data.card_type;
                if location != LOCATION_MZONE {
                    self.returns.set_i32(0, i32::from(positions));
                } else if card_type & TYPE_LINK != 0 && card_type & TYPE_MONSTER != 0 {
                    self.returns.set_i32(0, POS_FACEUP_ATTACK as i32);
                } else {
                    let code = self.cards[target].data.code;
                    self.emplace_process(SelectPosition {
                        playerid: move_player,
                        positions,
                        code,
                        ..SelectPosition::default()
                    });
                }
                false
            }
            2 => {
                let current = self.cards[target].current;
                if self.core.global_flag & GLOBALFLAG_DECK_REVERSE_CHECK != 0 && current.location == LOCATION_DECK {
                    let deck = &self.player[current.controler].list_main;
                    if current.sequence > 0
                        && current.sequence as usize + 1 == deck.len()
                        && let Some(&top) = deck.get(current.sequence as usize - 1)
                    {
                        let (code, position) = (self.cards[top].data.code, self.cards[top].current.position);
                        if self.core.deck_reversed || position == POS_FACEUP_DEFENSE {
                            self.trace
                                .message(MSG_DECK_TOP)
                                .u8(current.controler)
                                .u32(1)
                                .u32(code)
                                .u32(position);
                        }
                    }
                }
                let from = self.cards.get_info_location(target);
                if let Some(overlay_target) = self.cards[target].overlay_target {
                    self.xyz_remove(overlay_target, target);
                }
                let sequence = self.cards[target].temp.sequence as u8;
                self.move_card(playerid, target, location, sequence, arg.pzone);
                self.cards[target].current.position = self.returns.at_i32(0) as u32;
                self.cards[target].set_status(STATUS_LEAVE_CONFIRMED, false);
                let card = &self.cards[target];
                let (code, to, card_reason) = (card.data.code, self.cards.get_info_location(target), card.current.reason);
                self.trace.message(MSG_MOVE).u32(code).loc(from).loc(to).u32(card_reason);
                let equips = self.equiping_cards(target);
                if self.cards[target].current.location != LOCATION_MZONE && !equips.is_empty() {
                    let reason = REASON_LOST_TARGET + REASON_RULE;
                    self.field_destroy(equips, None, reason, PLAYER_NONE, PLAYER_NONE, LOCATION_GRAVE, 0);
                    for equip in equips {
                        self.card_unequip(equip);
                    }
                }
                if self.cards[target].previous.location == LOCATION_SZONE {
                    self.card_unequip(target);
                }
                let materials = self.xyz_materials(target);
                if self.cards[target].current.location != LOCATION_MZONE && !materials.is_empty() {
                    let reason = REASON_LOST_TARGET + REASON_RULE;
                    self.send_to(
                        materials,
                        None,
                        reason,
                        PLAYER_NONE,
                        PLAYER_NONE,
                        LOCATION_GRAVE,
                        0,
                        POS_FACEUP as u8,
                        false,
                    );
                }
                if self.cards[target].current.location == LOCATION_MZONE {
                    unported!(
                        self,
                        self.filter_player_effect(0, EFFECT_MUST_USE_MZONE, false)
                            .into_iter()
                            .chain(self.filter_player_effect(1, EFFECT_MUST_USE_MZONE, false))
                            .chain(self.filter_effect(target, EFFECT_MUST_USE_MZONE, true))
                    );
                    unported!(self, self.is_affected_by_effect(target, EFFECT_PRE_MONSTER));
                }
                if arg.enable {
                    self.enable_field_effect(target, true);
                }
                let current = self.cards[target].current;
                if self.cards[target].previous.location == LOCATION_GRAVE {
                    self.raise_single_event(target, EVENT_LEAVE_GRAVE, current.reason_effect, current.reason, move_player, 0, 0);
                    self.field_raise_event(
                        Some(target.into()),
                        EVENT_LEAVE_GRAVE,
                        current.reason_effect,
                        current.reason,
                        move_player,
                        0,
                        0,
                    );
                }
                self.raise_single_event(
                    target,
                    EVENT_MOVE,
                    current.reason_effect,
                    current.reason,
                    current.reason_player,
                    0,
                    0,
                );
                self.field_raise_event(
                    Some(target.into()),
                    EVENT_MOVE,
                    current.reason_effect,
                    current.reason,
                    current.reason_player,
                    0,
                    0,
                );
                self.process_single_event();
                self.process_instant_event();
                self.adjust_disable_check_list();
                false
            }
            3 => {
                self.returns.set_i32(0, 1);
                true
            }
            _ => true,
        }
    }

    // Step 1 sends trap monsters turned face-down back to the spell zone, and step 2 those without room to the GY;
    // trap monsters need effects (phase 2).
    pub(crate) fn process_change_pos(&mut self, arg: &mut ChangePos) -> bool {
        let (reason_effect, reason_player) = (arg.reason_effect, arg.reason_player);
        match arg.step {
            0 => {
                let targets = arg.targets;
                for card in targets.iter() {
                    let target = &self.cards[card];
                    let (npos, opos, card_type) = (target.position_param & 0xff, target.current.position, target.data.card_type);
                    if (target.current.location != LOCATION_MZONE && target.current.location != LOCATION_SZONE)
                        || (card_type & TYPE_LINK != 0 && card_type & TYPE_MONSTER != 0)
                        || target.get_status(STATUS_SUMMONING | STATUS_SPSUMMON_STEP)
                        || (reason_effect.is_some() && !self.is_affect_by_effect(card, reason_effect))
                        || npos == opos
                        || (card_type & TYPE_TOKEN == 0
                            && opos & POS_FACEUP != 0
                            && npos & POS_FACEDOWN != 0
                            && !self.is_capable_turn_set(card, reason_player))
                    {
                        arg.targets.remove(card);
                        continue;
                    }
                    if reason_effect.is_some() {
                        unported!(self, self.filter_effect(card, EFFECT_CANNOT_CHANGE_POS_E, true));
                    }
                }
                arg.to_grave_set.clear();
                false
            }
            1 => {
                arg.step = 2;
                false
            }
            3 => {
                if !arg.oppo_selection {
                    arg.oppo_selection = true;
                    arg.step = 0;
                }
                false
            }
            4 => {
                let mut cv: Cards16 = arg.targets.iter().collect();
                cv.sort_by(|&c1, &c2| self.card_operation_sort(c1, c2));
                let (mut flips, mut pos_changed, mut equipings) = (CardSet::default(), CardSet::default(), CardSet::default());
                for card in cv {
                    let target = &mut self.cards[card];
                    let (mut npos, opos, flag) = (target.position_param & 0xff, target.current.position, target.position_param >> 16);
                    if target.data.card_type & TYPE_TOKEN != 0 && npos & POS_FACEDOWN != 0 {
                        npos = POS_FACEUP_DEFENSE;
                    }
                    target.previous.position = opos;
                    target.current.position = npos;
                    if npos & POS_DEFENSE != 0 && self.is_affected_by_effect(card, EFFECT_DEFENSE_ATTACK).is_none() {
                        self.cards[card].set_status(STATUS_ATTACK_CANCELED, true);
                    }
                    self.cards[card].set_status(STATUS_JUST_POS, true);
                    let current = self.cards[card].current;
                    let mut message = self.trace.message(MSG_POS_CHANGE);
                    message
                        .u32(self.cards[card].data.code)
                        .u8(current.controler)
                        .u8(current.location)
                        .u8(current.sequence as u8);
                    message.u8(opos as u8).u8(npos as u8);
                    drop(message);
                    self.core.hint_timing[current.controler] |= TIMING_POS_CHANGE;
                    if opos & POS_FACEDOWN != 0 && npos & POS_FACEUP != 0 {
                        self.cards[card].fieldid = self.next_field_id();
                        unported!(self, self.check_unique_onfield(card, current.controler, current.location));
                        if current.location == LOCATION_MZONE {
                            self.raise_single_event(card, EVENT_FLIP, reason_effect, 0, reason_player, 0, flag);
                            flips.insert(card);
                        }
                        if !arg.enable {
                            self.refresh_disable_status(card);
                        } else if reason_effect.is_none_or(|effect| self.effects[effect].effect_type & 0x7f0 == 0)
                            || current.location != LOCATION_MZONE
                        {
                            self.enable_field_effect(card, true);
                        } else {
                            self.core.delayed_enable_set.insert(card);
                        }
                    }
                    if current.location == LOCATION_MZONE {
                        self.raise_single_event(card, EVENT_CHANGE_POS, reason_effect, 0, reason_player, 0, 0);
                        pos_changed.insert(card);
                    }
                    // Phase 2: trap monsters turned face-down move back to the spell zone.
                    if opos & POS_FACEUP != 0 && npos & POS_FACEDOWN != 0 {
                        self.cards[card].set_status(STATUS_SUMMON_DISABLED | STATUS_ACTIVATE_DISABLED, false);
                        self.reset(card, RESET_TURN_SET, RESET_EVENT);
                        self.clear_card_target(card);
                        self.cards[card].set_status(STATUS_SET_TURN, true);
                        self.enable_field_effect(card, false);
                        let target = &mut self.cards[card];
                        target.previous.location = 0;
                        target.summon.summon_type &= 0xdf00_ffff;
                        if target.summon.summon_type & SUMMON_TYPE_PENDULUM == SUMMON_TYPE_PENDULUM {
                            target.summon.summon_type &= 0xf000_ffff;
                        }
                    }
                    if npos & POS_FACEDOWN != 0 {
                        for equip in self.equiping_cards(card) {
                            equipings.insert(equip);
                            self.card_unequip(equip);
                        }
                        self.card_unequip(card);
                    }
                }
                self.adjust_instant();
                self.process_single_event();
                for (cards, event) in [(flips, EVENT_FLIP), (pos_changed, EVENT_CHANGE_POS)] {
                    if !cards.is_empty() {
                        self.field_raise_event(Some(cards), event, reason_effect, 0, reason_player, 0, 0);
                    }
                }
                self.process_instant_event();
                if !equipings.is_empty() {
                    let reason = REASON_LOST_TARGET + REASON_RULE;
                    self.field_destroy(equipings, None, reason, PLAYER_NONE, PLAYER_NONE, LOCATION_GRAVE, 0);
                }
                if !arg.to_grave_set.is_empty() {
                    let to_grave_set = std::mem::take(&mut arg.to_grave_set);
                    self.send_to(
                        to_grave_set,
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
            5 => {
                self.core.operated_set = arg.targets;
                self.returns.set_i32(0, arg.targets.len() as i32);
                true
            }
            _ => true,
        }
    }

    // SelectTributeP when a monster zone stays free, else one card at a time until a monster in a usable zone is
    // among the tributes.
    pub(crate) fn process_select_tribute(&mut self, arg: &mut SelectTribute) -> bool {
        let (target, playerid, toplayer) = (arg.target, arg.playerid, arg.toplayer);
        let forced_zones = self.get_forced_zones(target, toplayer, LOCATION_MZONE, playerid, LOCATION_REASON_TOFIELD);
        let zone = u32::from(arg.zone) & 0x1f & forced_zones;
        let (ct, _) = self.get_tofield_count(target, toplayer, LOCATION_MZONE, playerid, LOCATION_REASON_TOFIELD, zone);
        let (release_cards, release_cards_ex) = (self.core.release_cards, self.core.release_cards_ex);
        let (cancelable, min, max) = (arg.cancelable, arg.min as u8, arg.max as u8);
        let tribute_p = SelectTributeP {
            playerid,
            cancelable,
            min,
            max,
            ..SelectTributeP::default()
        };
        match arg.step {
            0 => {
                self.core.operated_set.clear();
                if ct > 0 {
                    self.trace.message(MSG_HINT).u8(HINT_SELECTMSG).u8(playerid).u64(500);
                    if release_cards_ex.len() + self.core.release_cards_ex_oneof.len() == 0 {
                        self.core.select_cards = release_cards.iter().collect();
                        self.emplace_process(tribute_p);
                        return true;
                    }
                    if release_cards_ex.len() >= usize::from(arg.max) {
                        self.core.select_cards = release_cards_ex.iter().collect();
                        self.emplace_process(tribute_p);
                        return true;
                    }
                }
                false
            }
            1 => {
                let mut must_choose_one = CardSet::default();
                let mut rmax = 0;
                for card in release_cards.iter() {
                    let current = self.cards[card].current;
                    if current.location == LOCATION_MZONE && current.controler == toplayer && (zone >> current.sequence) & 1 != 0 && ct <= 0
                    {
                        must_choose_one.insert(card);
                    }
                    rmax += self.cards[card].release_param;
                }
                self.trace.message(MSG_HINT).u8(HINT_SELECTMSG).u8(playerid).u64(500);
                if release_cards_ex.is_empty() && self.core.release_cards_ex_oneof.is_empty() && u32::from(arg.min) > rmax {
                    if rmax > 0 {
                        self.core.select_cards = release_cards.iter().collect();
                    }
                    self.emplace_process(tribute_p);
                    return true;
                }
                let force = !must_choose_one.is_empty();
                self.core.unselect_cards.clear();
                let oneof = self.core.release_cards_ex_oneof;
                for card in oneof.iter() {
                    if release_cards.contains(card) || release_cards_ex.contains(card) {
                        self.core.release_cards_ex_oneof.remove(card);
                    }
                }
                let oneof = self.core.release_cards_ex_oneof;
                self.core.select_cards = if i64::from(arg.max) - release_cards_ex.len() as i64 == 1 && force {
                    must_choose_one.iter().chain(release_cards_ex.iter()).collect()
                } else if arg.max <= 1 && force {
                    must_choose_one.iter().collect()
                } else {
                    release_cards.iter().chain(release_cards_ex.iter()).chain(oneof.iter()).collect()
                };
                self.emplace_process(SelectUnselectCard {
                    playerid,
                    cancelable,
                    min,
                    max,
                    ..SelectUnselectCard::default()
                });
                arg.step = 2;
                arg.must_choose_one = must_choose_one;
                false
            }
            2 => {
                let operated = self.core.operated_set;
                let (remaining_min, remaining_max) = self.remaining_tributes(arg);
                let must_choose_one = arg.must_choose_one;
                let force = !must_choose_one.is_empty() && !must_choose_one.iter().any(|card| operated.contains(card));
                let release_cards_ex = release_cards_ex.difference(operated);
                let exsize = release_cards_ex.len() as i64;
                let oneof = if arg.extra_release_effect.is_none() {
                    self.core.release_cards_ex_oneof
                } else {
                    CardSet::default()
                };
                self.core.select_cards = if i64::from(remaining_max) - exsize == 1 && force {
                    must_choose_one.difference(operated).iter().chain(release_cards_ex.iter()).collect()
                } else if remaining_max <= 1 && force {
                    must_choose_one.iter().collect()
                } else if exsize != 0 && exsize == i64::from(remaining_max) {
                    release_cards_ex.iter().collect()
                } else {
                    let (release_cards, oneof) = (release_cards.difference(operated), oneof.difference(operated));
                    release_cards.iter().chain(release_cards_ex.iter()).chain(oneof.iter()).collect()
                };
                self.core.unselect_cards = operated.iter().collect();
                self.trace.message(MSG_HINT).u8(HINT_SELECTMSG).u8(playerid).u64(500);
                let (cancelable, finishable) = (operated.is_empty() && cancelable, remaining_min == 0 && !force && exsize == 0);
                self.emplace_process(SelectUnselectCard {
                    playerid,
                    cancelable,
                    min,
                    max,
                    finishable,
                    ..SelectUnselectCard::default()
                });
                false
            }
            3 => {
                if self.return_cards.canceled && self.core.operated_set.is_empty() {
                    return true;
                }
                if !self.return_cards.canceled
                    && let Some(&card) = self.return_cards.list.first()
                {
                    if self.core.operated_set.contains(card) {
                        self.core.operated_set.remove(card);
                    } else {
                        self.core.operated_set.insert(card);
                    }
                    if self.core.release_cards_ex_oneof.contains(card) {
                        arg.extra_release_effect = self.is_affected_by_effect(card, EFFECT_EXTRA_RELEASE_SUM);
                    }
                }
                let (remaining_min, remaining_max) = self.remaining_tributes(arg);
                if (self.return_cards.canceled && remaining_min == 0) || remaining_max == 0 {
                    self.return_cards.clear();
                    self.return_cards.list = self.core.operated_set.iter().collect();
                    unported!(self, arg.extra_release_effect);
                    return true;
                }
                arg.step = 1;
                false
            }
            _ => true,
        }
    }

    // What the operated set still lacks: release count towards min and cards towards max.
    fn remaining_tributes(&self, arg: &SelectTribute) -> (u16, u16) {
        let operated = self.core.operated_set;
        let released: u32 = operated.iter().map(|card| self.cards[card].release_param).sum();
        (
            u32::from(arg.min).saturating_sub(released) as u16,
            (usize::from(arg.max).saturating_sub(operated.len())) as u16,
        )
    }
}

// `summon_proc_effect` is the procedure chosen, `proc_g` the group of an EFFECT_SPSUMMON_PROC_G procedure, and
// `procedure` the materials Duel.SynchroSummon and friends pass on, which the core keeps in core.must_use_mats and
// friends. Both are boxed to keep the unit small.
#[derive(Debug, Clone)]
pub struct SpSummonRule {
    pub step: u16,
    pub sumplayer: u8,
    pub target: CardId,
    pub summon_type: u32,
    pub summon_proc_effect: Option<EffectId>,
    pub proc_g: Box<ProcGroup>,
    pub procedure: Option<Box<Procedure>>,
}

// The cards of an EFFECT_SPSUMMON_PROC_G summon, and those not yet moved.
#[derive(Debug, Clone, Default)]
pub struct ProcGroup {
    pub cards: CardSet,
    pub remaining: CardSet,
}

impl Duel {
    pub(crate) fn special_summon_rule(&mut self, sumplayer: u8, target: CardId, summon_type: u32, procedure: Option<Procedure>) {
        self.emplace_process(SpSummonRule {
            step: 0,
            sumplayer,
            target,
            summon_type,
            summon_proc_effect: None,
            proc_g: Box::default(),
            procedure: procedure.map(Box::new),
        });
    }
    // Steps 0-17 summon by EFFECT_SPSUMMON_PROC, 20-28 the group of EFFECT_SPSUMMON_PROC_G.
    pub(crate) fn process_sp_summon_rule(&mut self, arg: &mut SpSummonRule) -> bool {
        let (sumplayer, target) = (arg.sumplayer, arg.target);
        let proc = arg.summon_proc_effect.unwrap_or_default();
        let nil_event = self.nil_event;
        match arg.step {
            0 => {
                let procedure = arg.procedure.as_deref().map_or(
                    Procedure {
                        card: target,
                        ..Procedure::default()
                    },
                    |&procedure| Procedure { card: target, ..procedure },
                );
                let mut procedures = self.filter_spsummon_procedure(target, sumplayer, arg.summon_type, procedure);
                procedures.extend(self.filter_spsummon_procedure_g(target, sumplayer));
                if procedures.is_empty() {
                    return true;
                }
                self.core.select_options = procedures.iter().map(|&effect| self.effects[effect].description).collect();
                self.core.select_effects = procedures.iter().copied().collect();
                if self.core.select_options.len() == 1 {
                    self.returns.set_i32(0, 0);
                } else {
                    self.emplace_process(SelectOption {
                        playerid: sumplayer,
                        ..SelectOption::default()
                    });
                }
                false
            }
            1 => {
                let selected = usize::try_from(self.returns.at_i32(0)).ok();
                let Some(&effect) = selected.and_then(|index| self.core.select_effects.get(index)) else {
                    return true;
                };
                arg.summon_proc_effect = Some(effect);
                if self.effects[effect].code == EFFECT_SPSUMMON_PROC_G {
                    arg.step = 19;
                    return false;
                }
                self.returns.set_i32(0, 1);
                if matches!(self.effects[effect].target, Target::Action(_)) {
                    let procedure = arg.procedure.as_deref().map_or(
                        Procedure {
                            card: target,
                            ..Procedure::default()
                        },
                        |&procedure| Procedure { card: target, ..procedure },
                    );
                    self.execute_procedure(ExecuteKind::Target, effect, sumplayer, nil_event, Some(procedure));
                }
                false
            }
            2 => {
                if self.returns.at_i32(0) == 0 {
                    return true;
                }
                unported!(self, self.filter_effect(target, EFFECT_SPSUMMON_COST, true));
                false
            }
            3 => {
                self.set_material_cards(target, CardSet::default());
                if self.effects[proc].operation.is_some() {
                    let procedure = arg.procedure.as_deref().map_or(
                        Procedure {
                            card: target,
                            ..Procedure::default()
                        },
                        |&procedure| Procedure { card: target, ..procedure },
                    );
                    self.execute_procedure(ExecuteKind::Operation, proc, sumplayer, nil_event, Some(procedure));
                }
                self.dec_count(proc, sumplayer);
                false
            }
            4 => {
                let peffect = &self.effects[proc];
                let (mut positions, mut targetplayer) = (POS_FACEUP as u8, sumplayer);
                if peffect.is_flag(EFFECT_FLAG_SPSUM_PARAM) {
                    positions = peffect.s_range as u8;
                    if peffect.o_range != 0 {
                        targetplayer = 1 - sumplayer;
                    }
                }
                if positions == 0 {
                    positions = POS_FACEUP_ATTACK as u8;
                }
                let values = self.get_values(proc, target);
                let sumtype = (values.first().map_or(0, |&value| value as u32) & 0xf00_ffff) | SUMMON_TYPE_SPECIAL;
                let zone = values.get(1).map_or(0xff, |&value| value as u32);
                let card = &mut self.cards[target];
                card.summon.summon_type = sumtype;
                card.summon.location = card.current.location;
                card.summon.sequence = card.current.sequence;
                card.summon.pzone = card.current.pzone;
                self.enable_field_effect(target, false);
                unported!(self, self.filter_player_effect(sumplayer, EFFECT_FORCE_SPSUMMON_POSITION, true));
                self.field_move_to_field(
                    target,
                    sumplayer,
                    targetplayer,
                    LOCATION_MZONE.into(),
                    positions,
                    false,
                    zone as u8,
                    true,
                    LOCATION_REASON_SPSUMMON,
                );
                let card = &mut self.cards[target];
                card.current.reason = REASON_SPSUMMON;
                card.current.reason_effect = Some(proc);
                card.current.reason_player = sumplayer;
                card.summon.player = sumplayer;
                self.field_break_effect(true);
                false
            }
            5 => {
                self.shuffle_checked();
                let controler = self.cards[target].current.controler;
                self.set_control(target, controler, 0, 0);
                self.core.phase_action = true;
                self.cards[target].current.reason_effect = Some(proc);
                self.write_spsummoning(target);
                false
            }
            6 => {
                self.raise_material_event(EVENT_BE_PRE_MATERIAL, proc, target, sumplayer);
                false
            }
            7 => {
                let disablable =
                    self.core.current_chain.is_empty() && self.is_affected_by_effect(target, EFFECT_CANNOT_DISABLE_SPSUMMON).is_none();
                arg.step = if disablable { 9 } else { 14 };
                false
            }
            10 => {
                self.cards[target].set_status(STATUS_SUMMONING, true);
                self.cards[target].set_status(STATUS_SUMMON_DISABLED, false);
                self.field_raise_event(Some(target.into()), EVENT_SPSUMMON, Some(proc), 0, sumplayer, sumplayer, 0);
                self.process_instant_event();
                self.emplace_process(PointEvent {
                    skip_trigger: true,
                    skip_freechain: true,
                    skip_new: true,
                    ..PointEvent::default()
                });
                false
            }
            11 => {
                if self.cards[target].is_status(STATUS_SUMMONING) {
                    arg.step = 14;
                    return false;
                }
                self.remove_procedure_oath(proc, sumplayer);
                if self.cards[target].current.location == LOCATION_MZONE {
                    self.send_to(target.into(), None, REASON_RULE, sumplayer, sumplayer, LOCATION_GRAVE, 0, 0, false);
                }
                self.adjust_instant();
                self.emplace_process(PointEvent::default());
                true
            }
            15 => {
                self.release_oath_relation(proc);
                self.cards[target].set_status(STATUS_SUMMONING, false);
                self.cards[target].set_status(STATUS_PROC_COMPLETE | STATUS_SPSUMMON_TURN, true);
                self.enable_field_effect(target, true);
                if self.cards[target].is_status(STATUS_DISABLED) {
                    self.reset(target, RESET_DISABLE, RESET_EVENT);
                }
                false
            }
            16 => {
                self.trace.message(MSG_SPSUMMONED);
                self.adjust_instant();
                self.raise_material_event(EVENT_BE_MATERIAL, proc, target, sumplayer);
                false
            }
            17 => {
                self.core.spsummon_state_count[sumplayer] += 1;
                self.raise_single_event(target, EVENT_SPSUMMON_SUCCESS, Some(proc), 0, sumplayer, sumplayer, 0);
                self.process_single_event();
                self.field_raise_event(Some(target.into()), EVENT_SPSUMMON_SUCCESS, Some(proc), 0, sumplayer, sumplayer, 0);
                self.process_instant_event();
                self.point_event_after_summon(sumplayer);
                true
            }
            20 => {
                self.core.cards_to_summon_g.clear();
                if self.effects[proc].operation.is_some() {
                    self.execute_procedure(
                        ExecuteKind::Operation,
                        proc,
                        sumplayer,
                        nil_event,
                        Some(Procedure {
                            card: target,
                            ..Procedure::default()
                        }),
                    );
                }
                self.dec_count(proc, sumplayer);
                false
            }
            21 => {
                let mut group = self.core.cards_to_summon_g;
                for card in group {
                    let current = self.cards[card].current;
                    if self.cards[card].data.card_type & TYPE_MONSTER == 0
                        || current.location == LOCATION_MZONE
                        || self.check_unique_onfield(card, sumplayer, LOCATION_MZONE).is_some()
                        || self.is_affected_by_effect(card, EFFECT_CANNOT_SPECIAL_SUMMON).is_some()
                    {
                        group.remove(card);
                        continue;
                    }
                    unported!(self, self.filter_effect(card, EFFECT_SPSUMMON_COST, true));
                }
                arg.proc_g.cards = group;
                false
            }
            22 => {
                if arg.proc_g.cards.is_empty() {
                    return true;
                }
                self.core.phase_action = true;
                arg.proc_g.remaining = arg.proc_g.cards;
                false
            }
            23 => {
                let Some(card) = arg.proc_g.remaining.first() else { return true };
                self.enable_field_effect(card, false);
                let sumtype = (self.get_value(proc, card) as u32 & 0xff00_ffff) | SUMMON_TYPE_SPECIAL;
                let target = &mut self.cards[card];
                target.current.reason = REASON_SPSUMMON;
                target.current.reason_effect = Some(proc);
                target.current.reason_player = sumplayer;
                target.summon.player = sumplayer;
                target.summon.summon_type = sumtype;
                target.summon.location = target.current.location;
                target.summon.sequence = target.current.sequence;
                target.summon.pzone = target.current.pzone;
                unported!(self, self.filter_player_effect(sumplayer, EFFECT_FORCE_SPSUMMON_POSITION, true));
                let reason = LOCATION_REASON_TOFIELD;
                let (mut main, main_flag) = self.get_tofield_count(Some(card), sumplayer, LOCATION_MZONE, sumplayer, reason, 0xff);
                let (mut extra, extra_flag) = self.get_spsummonable_count_fromex(Some(card), sumplayer, sumplayer, 0xff);
                for other in arg.proc_g.remaining {
                    if self.cards[other].current.location == LOCATION_EXTRA {
                        extra -= 1;
                    } else {
                        main -= 1;
                    }
                }
                let zone = match self.cards[card].current.location {
                    LOCATION_EXTRA if main == 0 => main_flag,
                    LOCATION_EXTRA => 0xff,
                    _ if extra == 0 => extra_flag,
                    _ => 0xff,
                };
                self.field_move_to_field(
                    card,
                    sumplayer,
                    sumplayer,
                    LOCATION_MZONE.into(),
                    POS_FACEUP as u8,
                    false,
                    zone as u8,
                    true,
                    LOCATION_REASON_SPSUMMON,
                );
                false
            }
            24 => {
                let Some(card) = arg.proc_g.remaining.first() else { return true };
                arg.proc_g.remaining.remove(card);
                self.write_spsummoning(card);
                let controler = self.cards[card].current.controler;
                self.set_control(card, controler, 0, 0);
                self.cards[card].set_status(STATUS_SPSUMMON_STEP, true);
                if !arg.proc_g.remaining.is_empty() {
                    arg.step = 22;
                }
                false
            }
            25 => {
                let group = arg.proc_g.cards;
                if !self.core.current_chain.is_empty() {
                    for card in group {
                        self.cards[card].set_status(STATUS_SPSUMMON_STEP, false);
                    }
                    arg.step = 26;
                    return false;
                }
                let mut disablable = CardSet::default();
                for card in group {
                    self.cards[card].set_status(STATUS_SUMMONING, true);
                    self.cards[card].set_status(STATUS_SPSUMMON_STEP, false);
                    if self.is_affected_by_effect(card, EFFECT_CANNOT_DISABLE_SPSUMMON).is_none() {
                        disablable.insert(card);
                    }
                }
                if !disablable.is_empty() {
                    self.field_raise_event(Some(disablable), EVENT_SPSUMMON, Some(proc), 0, sumplayer, sumplayer, 0);
                    self.process_instant_event();
                    self.emplace_process(PointEvent {
                        skip_trigger: true,
                        skip_freechain: true,
                        skip_new: true,
                        ..PointEvent::default()
                    });
                }
                false
            }
            26 => {
                let mut negated = CardSet::default();
                for card in arg.proc_g.cards {
                    if !self.cards[card].is_status(STATUS_SUMMONING) {
                        arg.proc_g.cards.remove(card);
                        if self.cards[card].current.location == LOCATION_MZONE {
                            negated.insert(card);
                        }
                    }
                }
                if !negated.is_empty() {
                    self.send_to(negated, None, REASON_RULE, sumplayer, sumplayer, LOCATION_GRAVE, 0, 0, false);
                    self.adjust_instant();
                }
                if arg.proc_g.cards.is_empty() {
                    self.remove_procedure_oath(proc, sumplayer);
                    self.emplace_process(PointEvent::default());
                    return true;
                }
                false
            }
            27 => {
                self.release_oath_relation(proc);
                for card in arg.proc_g.cards {
                    self.cards[card].set_status(STATUS_SUMMONING, false);
                    self.cards[card].set_status(STATUS_SPSUMMON_TURN, true);
                    self.enable_field_effect(card, true);
                    if self.cards[card].is_status(STATUS_DISABLED) {
                        self.reset(card, RESET_DISABLE, RESET_EVENT);
                    }
                }
                false
            }
            28 => {
                let group = arg.proc_g.cards;
                self.trace.message(MSG_SPSUMMONED);
                self.core.spsummon_state_count[sumplayer] += 1;
                for card in group {
                    let current = self.cards[card].current;
                    let summon_player = self.cards[card].summon.player;
                    self.raise_single_event(
                        card,
                        EVENT_SPSUMMON_SUCCESS,
                        current.reason_effect,
                        0,
                        current.reason_player,
                        summon_player,
                        0,
                    );
                }
                self.process_single_event();
                self.field_raise_event(Some(group), EVENT_SPSUMMON_SUCCESS, Some(proc), 0, sumplayer, sumplayer, 0);
                self.process_instant_event();
                self.point_event_after_summon(sumplayer);
                true
            }
            _ => true,
        }
    }

    // The shuffles a summon procedure's checks left pending.
    fn shuffle_checked(&mut self) {
        for (location, checks) in [
            (LOCATION_HAND, self.core.shuffle_hand_check),
            (LOCATION_DECK, self.core.shuffle_deck_check),
        ] {
            for p in 0..2 {
                if checks[p] {
                    self.shuffle(p, location);
                }
            }
        }
    }

    fn write_spsummoning(&mut self, card: CardId) {
        let target = &self.cards[card];
        let code = if target.current.position & POS_FACEDOWN == 0 {
            target.data.code
        } else {
            0
        };
        let location = self.cards.get_info_location(card);
        self.trace.message(MSG_SPSUMMONING).u32(code).loc(location);
    }

    // EVENT_BE_PRE_MATERIAL and EVENT_BE_MATERIAL of the summoned monster's materials; the core raises the event with
    // the empty group for a procedure without materials all the same. The reason compares the procedure's value as an
    // integer, so a value function counts as a plain special summon.
    fn raise_material_event(&mut self, code: u32, proc: EffectId, target: CardId, sumplayer: u8) {
        let reason = match self.effects[proc].value {
            Value::Int(value) if value as u32 == SUMMON_TYPE_SYNCHRO => REASON_SYNCHRO,
            Value::Int(value) if value as u32 == SUMMON_TYPE_XYZ => REASON_XYZ,
            Value::Int(value) if value as u32 == SUMMON_TYPE_LINK => REASON_LINK,
            _ => REASON_SPSUMMON,
        };
        let materials = self.material_cards(target);
        for material in materials {
            self.raise_single_event(material, code, Some(proc), reason, sumplayer, sumplayer, 0);
        }
        self.field_raise_event(Some(materials), code, Some(proc), reason, sumplayer, sumplayer, 0);
        self.process_single_event();
        self.process_instant_event();
    }

    // A procedure summon negated: its oath effects go, and an oath count limit is given back.
    fn remove_procedure_oath(&mut self, proc: EffectId, sumplayer: u8) {
        self.remove_oath_effect(proc);
        let peffect = &self.effects[proc];
        if peffect.is_flag(EFFECT_FLAG_COUNT_LIMIT) && peffect.count_flag & EFFECT_COUNT_CODE_OATH != 0 {
            let (code, flag, hopt) = (peffect.count_code, peffect.count_flag, peffect.count_hopt_index);
            self.dec_effect_code(code, flag, hopt, sumplayer);
        }
    }

    fn point_event_after_summon(&mut self, sumplayer: u8) {
        if self.core.current_chain.is_empty() {
            self.adjust_all();
            self.core.hint_timing[sumplayer] |= TIMING_SPSUMMON;
            self.emplace_process(PointEvent::default());
        }
    }
}

// Counters removed from `card`, or chosen among the cards of the sides `self_side` and `oppo` (SelectCounter, inline:
// the prompt only comes up with counters on more than one card).
#[derive(Debug, Clone, Default)]
pub struct RemoveCounter {
    pub step: u16,
    pub reason: u32,
    pub card: Option<CardId>,
    pub rplayer: u8,
    pub self_side: u8,
    pub oppo: u8,
    pub counter_type: u16,
    pub count: u16,
}

impl Duel {
    pub(crate) fn field_remove_counter(
        &mut self,
        reason: u32,
        card: Option<CardId>,
        rplayer: u8,
        self_side: u8,
        oppo: u8,
        counter_type: u16,
        count: u16,
    ) {
        self.emplace_process(RemoveCounter {
            step: 0,
            reason,
            card,
            rplayer,
            self_side,
            oppo,
            counter_type,
            count,
        });
    }

    // Only the removal itself is an option until a card registers EFFECT_RCOUNTER_REPLACE.
    pub(crate) fn process_remove_counter(&mut self, arg: &mut RemoveCounter) -> bool {
        let (card, counter_type, count) = (arg.card, arg.counter_type, arg.count);
        match arg.step {
            0 => {
                let available = match card {
                    Some(card) => self.get_counter(card, counter_type) >= count,
                    None => self.get_field_counter(arg.rplayer, arg.self_side, arg.oppo, counter_type) != 0,
                };
                unported!(
                    self,
                    self.field_effect_range(Container::Continuous, EFFECT_RCOUNTER_REPLACE + u32::from(counter_type))
                );
                self.returns.set_i32(0, 0);
                !available
            }
            1 => {
                if let Some(card) = card {
                    let removed = self.card_remove_counter(card, counter_type, count);
                    self.returns.set_i32(0, i32::from(removed));
                    arg.step = 3;
                    return false;
                }
                if count == 0 {
                    return false;
                }
                let mut cards = Cards16::default();
                let mut total = 0;
                for (player, included) in [(arg.rplayer, arg.self_side), (1 - arg.rplayer, arg.oppo)] {
                    if included == 0 {
                        continue;
                    }
                    let info = &self.player[player];
                    for other in info.list_mzone.into_iter().chain(info.list_szone).flatten() {
                        let counters = self.get_counter(other, counter_type);
                        if counters != 0 {
                            cards.push(other);
                            total += counters;
                        }
                    }
                }
                self.core.select_cards = cards;
                if self.core.select_cards.len() == 1 {
                    self.returns.set_i16(0, count.min(total) as i16);
                } else {
                    self.reach_unported(concat!(file!(), ":", line!(), " (MSG_SELECT_COUNTER)"));
                }
                false
            }
            2 => {
                let cards = self.core.select_cards.clone();
                for (index, &other) in cards.iter().enumerate() {
                    let removed = self.returns.at_i16(index);
                    if removed > 0 {
                        self.card_remove_counter(other, counter_type, removed as u16);
                    }
                }
                false
            }
            3 => {
                let reason_effect = self.core.reason_effect.get();
                let code = EVENT_REMOVE_COUNTER + u32::from(counter_type);
                self.field_raise_event(None, code, reason_effect, arg.reason, arg.rplayer, arg.rplayer, u32::from(count));
                self.process_instant_event();
                false
            }
            4 => {
                self.returns.set_i32(0, 1);
                true
            }
            _ => true,
        }
    }
}

// Xyz materials a player detaches from the cards of `group`, or from the monsters on the sides `self_side` and `oppo`.
// The core offers EFFECT_OVERLAY_REMOVE_REPLACE effects in step 0, which come with the cards that register them.
#[derive(Debug, Clone, Default)]
pub struct RemoveOverlay {
    pub step: u16,
    pub reason: u32,
    pub group: Option<CardSet>,
    pub rplayer: u8,
    pub self_side: u8,
    pub oppo: u8,
    pub min: u16,
    pub max: u16,
}

// Duel.Overlay: `materials` become Xyz materials of `target`, and so do their own materials unless
// `send_materials_to_grave` sends those to the GY first.
#[derive(Debug, Clone, Default)]
pub struct XyzOverlay {
    pub step: u16,
    pub send_materials_to_grave: bool,
    pub target: CardId,
    pub materials: CardSet,
}

impl Duel {
    pub(crate) fn field_remove_overlay_card(
        &mut self,
        reason: u32,
        group: Option<CardSet>,
        rplayer: u8,
        self_side: u8,
        oppo: u8,
        min: u16,
        max: u16,
    ) {
        self.emplace_process(RemoveOverlay {
            step: 0,
            reason,
            group,
            rplayer,
            self_side,
            oppo,
            min,
            max,
        });
    }

    pub(crate) fn xyz_overlay(&mut self, target: CardId, materials: CardSet, send_materials_to_grave: bool) {
        self.emplace_process(XyzOverlay {
            step: 0,
            send_materials_to_grave,
            target,
            materials,
        });
    }

    pub(crate) fn process_remove_overlay(&mut self, arg: &mut RemoveOverlay) -> bool {
        let (rplayer, reason) = (arg.rplayer, arg.reason);
        match arg.step {
            0 => {
                let available = self.field_overlay_group(rplayer, arg.self_side, arg.oppo, arg.group).len() >= usize::from(arg.min);
                unported!(self, self.field_effect_range(Container::Continuous, EFFECT_OVERLAY_REMOVE_REPLACE));
                self.returns.set_i32(0, 0);
                !available
            }
            1 => false,
            2 => {
                let cards = self.field_overlay_group(rplayer, arg.self_side, arg.oppo, arg.group);
                self.core.select_cards = cards.iter().collect();
                self.trace.message(MSG_HINT).u8(HINT_SELECTMSG).u8(rplayer).u64(HINTMSG_REMOVEXYZ);
                self.emplace_process(SelectCard {
                    playerid: rplayer,
                    cancelable: false,
                    min: arg.min as u8,
                    max: arg.max as u8,
                    ..SelectCard::default()
                });
                false
            }
            3 => {
                let cards = self.return_cards.list.iter().copied().collect();
                let reason_effect = self.core.reason_effect.get();
                self.send_to(
                    cards,
                    reason_effect,
                    reason,
                    rplayer,
                    PLAYER_NONE,
                    LOCATION_GRAVE,
                    0,
                    POS_FACEUP as u8,
                    false,
                );
                false
            }
            _ => true,
        }
    }

    pub(crate) fn process_xyz_overlay(&mut self, arg: &mut XyzOverlay) -> bool {
        let (target, materials) = (arg.target, arg.materials);
        match arg.step {
            0 => {
                for card in materials.iter() {
                    if self.cards[card].overlay_target.is_none() {
                        self.enable_field_effect(card, false);
                    }
                }
                if !arg.send_materials_to_grave {
                    return false;
                }
                let to_grave = materials
                    .iter()
                    .fold(CardSet::default(), |to_grave, card| to_grave.union(self.xyz_materials(card)));
                let reason = REASON_RULE + REASON_LOST_TARGET;
                self.send_to(
                    to_grave,
                    None,
                    reason,
                    PLAYER_NONE,
                    PLAYER_NONE,
                    LOCATION_GRAVE,
                    0,
                    POS_FACEUP as u8,
                    false,
                );
                false
            }
            1 => {
                let sort = |duel: &Self, cards: &mut [CardId]| cards.sort_by(|&c1, &c2| duel.card_operation_sort(c1, c2));
                let mut others: Cards16 = materials
                    .iter()
                    .filter(|&card| self.cards[card].overlay_target != Some(target))
                    .collect();
                // The materials' own materials go in front, those of the last material by card id first.
                let mut cv = Cards16::default();
                if !arg.send_materials_to_grave {
                    let holders: Cards16 = materials.iter().collect();
                    for &card in holders.iter().rev() {
                        let mut attached: Cards16 = self.xyz_materials(card).iter().collect();
                        attached.sort_by_key(|&material| self.cards[material].current.sequence);
                        cv.extend(attached);
                    }
                    // The core leaves the last of them out of its sort.
                    if let Some(head) = cv.len().checked_sub(1).and_then(|end| cv.get_mut(..end)) {
                        sort(self, head);
                    }
                    if others.len() >= 2 {
                        sort(self, &mut others);
                    }
                }
                cv.extend(others);
                unported!(self, (self.core.global_flag & GLOBALFLAG_DECK_REVERSE_CHECK != 0).then_some(()));
                let (reason_effect, reason_player) = (self.core.reason_effect.get(), self.core.reason_player.get());
                let (mut from_grave, mut des) = (CardSet::default(), CardSet::default());
                for card in cv {
                    self.cards[card].current.reason = REASON_XYZ + REASON_MATERIAL;
                    self.reset(card, RESET_LEAVE + RESET_OVERLAY, RESET_EVENT);
                    self.card_unequip(card);
                    for equip in self.equiping_cards(card) {
                        des.insert(equip);
                        self.card_unequip(equip);
                    }
                    self.clear_card_target(card);
                    let from = self.cards.get_info_location(card);
                    if let Some(overlay_target) = self.cards[card].overlay_target {
                        self.xyz_remove(overlay_target, card);
                    } else {
                        self.remove_card(card);
                        self.add_to_disable_check_list(card);
                        if self.cards[card].previous.location == LOCATION_GRAVE {
                            from_grave.insert(card);
                            self.raise_single_event(card, EVENT_LEAVE_GRAVE, reason_effect, 0, reason_player, 0, 0);
                        }
                    }
                    self.xyz_add(target, card);
                    let moved = &self.cards[card];
                    let (code, to, reason) = (moved.data.code, self.cards.get_info_location(card), moved.current.reason);
                    self.trace.message(MSG_MOVE).u32(code).loc(from).loc(to).u32(reason);
                }
                if !from_grave.is_empty() {
                    self.field_raise_event(Some(from_grave), EVENT_LEAVE_GRAVE, reason_effect, 0, reason_player, 0, 0);
                    self.process_single_event();
                    self.process_instant_event();
                }
                if des.is_empty() {
                    self.adjust_instant();
                } else {
                    self.field_destroy(
                        des,
                        None,
                        REASON_LOST_TARGET + REASON_RULE,
                        PLAYER_NONE,
                        PLAYER_NONE,
                        LOCATION_GRAVE,
                        0,
                    );
                }
                if self.cards[target].current.location & LOCATION_ONFIELD != 0 {
                    self.adjust_all();
                }
                false
            }
            _ => true,
        }
    }
}

// Duel.SSet: the cards are placed one zone at a time, then set together. The core keeps the placed cards and zones in
// core.set_group_*; here they are the unit's own, boxed to keep the unit small.
#[derive(Debug, Clone)]
pub struct SpellSetGroup {
    pub step: u16,
    pub setplayer: u8,
    pub toplayer: u8,
    pub confirm: bool,
    pub reason_effect: Option<EffectId>,
    pub state: Box<SpellSetGroupState>,
}

#[derive(Debug, Clone, Default)]
pub struct SpellSetGroupState {
    pub targets: CardSet,
    pub set_cards: CardSet,
    pub pre_set: CardSet,
    pub set: CardSet,
    pub used_zones: u32,
    pub seq: [u8; 7],
}

impl Duel {
    pub(crate) fn sset_group(&mut self, setplayer: u8, toplayer: u8, targets: CardSet, confirm: bool, reason_effect: Option<EffectId>) {
        self.emplace_process(SpellSetGroup {
            step: 0,
            setplayer,
            toplayer,
            confirm,
            reason_effect,
            state: Box::new(SpellSetGroupState {
                targets,
                ..SpellSetGroupState::default()
            }),
        });
    }

    pub(crate) fn process_spell_set_group(&mut self, arg: &mut SpellSetGroup) -> bool {
        let (setplayer, toplayer) = (arg.setplayer, arg.toplayer);
        let state = &mut *arg.state;
        match arg.step {
            0 => {
                self.core.operated_set.clear();
                for target in state.targets {
                    let card = &self.cards[target];
                    let data_type = card.data.card_type;
                    if (data_type & TYPE_FIELD == 0
                        && self
                            .get_useable_count(Some(target), toplayer, LOCATION_SZONE, setplayer, LOCATION_REASON_TOFIELD, 0xff)
                            .0
                            <= 0)
                        || (data_type & TYPE_MONSTER != 0 && self.is_affected_by_effect(target, EFFECT_MONSTER_SSET).is_none())
                        || card.current.location == LOCATION_SZONE
                        || !self.is_player_can_sset(setplayer, target)
                        || self.is_affected_by_effect(target, EFFECT_CANNOT_SSET).is_some()
                    {
                        continue;
                    }
                    state.set_cards.insert(target);
                }
                if state.set_cards.is_empty() {
                    self.returns.set_i32(0, 0);
                    return true;
                }
                for card in state.set_cards {
                    unported!(self, self.filter_effect(card, EFFECT_SSET_COST, true));
                }
                self.core.phase_action = true;
                false
            }
            1 => {
                let Some(target) = state.set_cards.first() else { return true };
                if self.cards[target].data.card_type & TYPE_FIELD != 0 {
                    self.returns.set_u8(2, 5);
                    return false;
                }
                let (_, flag) = self.get_useable_count(Some(target), toplayer, LOCATION_SZONE, setplayer, LOCATION_REASON_TOFIELD, 0xff);
                let flag = flag | state.used_zones;
                let flag = if setplayer == toplayer {
                    ((flag & 0xff) << 8) | 0xffff_00ff
                } else {
                    ((flag & 0xff) << 24) | 0xff_ffff
                } | 0xe080_e080;
                let code = self.cards[target].data.code;
                self.trace.message(MSG_HINT).u8(HINT_SELECTMSG).u8(setplayer).u64(u64::from(code));
                self.emplace_process(SelectPlace {
                    playerid: setplayer,
                    count: 1,
                    flag,
                    ..SelectPlace::default()
                });
                false
            }
            2 => {
                let Some(target) = state.set_cards.first() else { return true };
                let seq = self.returns.at_u8(2);
                if let Some(slot) = state.seq.get_mut(state.pre_set.len()) {
                    *slot = seq;
                }
                state.pre_set.insert(target);
                state.used_zones |= 1 << seq;
                state.set_cards.remove(target);
                if !state.set_cards.is_empty() {
                    arg.step = 0;
                }
                false
            }
            3 => {
                let Some(target) = state.pre_set.first() else { return true };
                self.enable_field_effect(target, false);
                let zone = if self.cards[target].data.card_type & TYPE_FIELD != 0 {
                    1 << 5
                } else {
                    match (0..7).map(|i| 1u32 << i).find(|&zone| state.used_zones & zone != 0) {
                        Some(zone) => {
                            state.used_zones &= !zone;
                            zone
                        }
                        None => 1 << 6,
                    }
                };
                let szone = u16::from(LOCATION_SZONE);
                let unit = MoveToField {
                    step: 0,
                    enable: false,
                    pzone: false,
                    zone: zone as u8,
                    rule: false,
                    location_reason: LOCATION_REASON_NONE,
                    confirm: false,
                    ret: 0,
                    target,
                };
                self.move_to_field_unit(setplayer, toplayer, szone, POS_FACEDOWN as u8, unit);
                false
            }
            4 => {
                let Some(target) = state.pre_set.first() else { return true };
                self.cards[target].set_status(STATUS_SET_TURN, true);
                unported!(self, (self.cards[target].data.card_type & TYPE_MONSTER != 0).then_some(target));
                let (code, location) = (self.cards[target].data.code, self.cards.get_info_location(target));
                self.trace.message(MSG_SET).u32(code).loc(location);
                state.set.insert(target);
                state.pre_set.remove(target);
                if !state.pre_set.is_empty() {
                    arg.step = 2;
                }
                false
            }
            5 => {
                if arg.confirm {
                    let set: Vec<_> = state
                        .set
                        .iter()
                        .map(|card| (self.cards[card].data.code, self.cards[card].current))
                        .collect();
                    let mut message = self.trace.message(MSG_CONFIRM_CARDS);
                    message.u8(toplayer).u32(set.len() as u32);
                    for (code, current) in set {
                        message.u32(code).u8(current.controler).u8(current.location).u32(current.sequence);
                    }
                }
                false
            }
            6 => {
                self.core.operated_set = state.set;
                let mut ct = state.set.len() as u8;
                if state.used_zones & (1 << 5) != 0 {
                    ct -= 1;
                }
                if ct <= 1 {
                    return false;
                }
                unported!(self, Some(ct));
                false
            }
            7 => {
                self.adjust_instant();
                let operated = self.core.operated_set;
                self.field_raise_event(Some(operated), EVENT_SSET, arg.reason_effect, 0, setplayer, setplayer, 0);
                self.process_instant_event();
                if self.core.current_chain.is_empty() {
                    self.adjust_all();
                    self.core.hint_timing[setplayer] |= TIMING_SSET;
                    self.emplace_process(PointEvent::default());
                }
                false
            }
            8 => {
                self.returns.set_i32(0, self.core.operated_set.len() as i32);
                true
            }
            _ => true,
        }
    }
}
