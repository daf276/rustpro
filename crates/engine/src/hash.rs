use std::cell::{Cell, RefCell};
use std::hash::{BuildHasher, Hash, Hasher};

use rustc_hash::{FxBuildHasher, FxHasher};

use crate::action::Prompt;
use crate::card::{CardId, CardSet, MAX_CARDS};
use crate::chain::{Continuous, ExecuteKind};
use crate::constants::*;
use crate::duel::Duel;
use crate::effect::{Chain, Effect, EffectId, Event, OpInfos, Operation, Value};
use crate::field::Processor;
use crate::list::List;
use crate::{chain, operations, processor, script};

// Fx with a murmur3 finish, so that parts summed in any order still spread over all bits.
fn mix(part: impl Hash) -> u64 {
    let mut hash = FxBuildHasher.hash_one(part);
    hash = (hash ^ hash >> 33).wrapping_mul(0xff51_afd7_ed55_8ccd);
    hash = (hash ^ hash >> 33).wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    hash ^ hash >> 33
}

// The parts of an ordered list in order.
fn chained(parts: impl IntoIterator<Item = u64>) -> u64 {
    parts.into_iter().fold(0, |hash, part| mix((hash, part)))
}

fn summed(parts: impl IntoIterator<Item = u64>) -> u64 {
    parts.into_iter().fold(0, u64::wrapping_add)
}

// 0 for an index in the hand or the Extra Deck, whose order the board leaves out too.
fn field_sequence(location: u32, sequence: u32) -> u32 {
    match location & u32::from(LOCATION_HAND | LOCATION_EXTRA) {
        0 => sequence,
        _ => 0,
    }
}

// An initial effect as the card's script registered it, which the card's code already stands for.
fn is_as_registered(effect: &Effect) -> bool {
    effect.is_flag(EFFECT_FLAG_INITIAL)
        && effect.count_limit == effect.count_limit_max
        && effect.label_len == 0
        && effect.label_object.is_none()
}

impl Duel {
    // The same for the same state reached by another order of actions, as it hashes cards and effects by content, not by an
    // id, field id or timestamp; Some at every prompt where no script function waits. At prompts other than the idle one it
    // adds the processor's state (units, chain, events, pending triggers, battle records, the prompt's options and picks so
    // far), the cards and effects it refers to keyed by [Refs], with effects' function addresses, so those hashes compare
    // only within one process. What it leaves out (functions of registered effects, effect statuses, initial effects as
    // registered, battle records at the idle prompt) can merge distinct states.
    pub fn state_hash(&self) -> Option<u64> {
        let prompt = self.prompt()?;
        if self.replay.snapshot.is_some() {
            return None;
        }
        let refs = (!matches!(prompt, Prompt::IdleCommand { .. })).then(|| Refs::new(self));
        let (infos, core) = (&self.infos, &self.core);
        let players = self
            .player
            .0
            .each_ref()
            .map(|info| (info.lp, info.draw_count, info.disabled_location));
        let turn = (
            infos.turn_id,
            infos.turn_id_by_player,
            infos.phase,
            infos.turn_player,
            infos.can_shuffle,
            players,
        );
        let activity = (
            core.summon_count,
            core.extra_summon,
            core.spe_effect,
            core.summon_state_count,
            core.normalsummon_state_count,
            core.flipsummon_state_count,
            core.spsummon_state_count,
            core.spsummon_state_count_rst,
            core.attack_state_count,
            core.battle_phase_count,
            core.battled_count,
            (core.spsummon_rst, core.global_flag, core.current_chain.len()),
        );
        let count_codes = [&core.effect_count_code, &core.effect_count_code_duel, &core.effect_count_code_chain];
        let count_codes = (count_codes.into_iter().enumerate())
            .flat_map(|(map, codes)| codes.iter().filter(|(_, count)| *count > 0).map(move |entry| mix((map, entry))));
        let once = (0..2u8).flat_map(|player| core.spsummon_once_map[player].iter().map(move |entry| mix((player, entry))));
        let activities = (core.custom_counters.iter()).map(|counter| mix((counter.id, counter.activity, counter.amount)));
        let materials = self.material_cards.iter().map(|&(card, materials)| {
            let codes: u64 = materials.iter().map(|material| u64::from(self.cards[material].data.code)).sum();
            mix((self.place_key(card), codes))
        });
        let counters = (self.counters.iter()).map(|&(card, kind, counts)| mix((self.place_key(card), kind, counts)));
        let targets = (self.card_targets.iter()).map(|&(card, target)| mix((self.place_key(card), self.place_key(target))));
        let relations = (self.card_relations.iter()).map(|&(card, other, reset)| mix((self.place_key(card), self.place_key(other), reset)));
        let spent = (self.indestructable_effects.iter()).map(|&(card, _)| mix(self.place_key(card)));
        let script_groups = self.script_groups.iter().map(|&(key, cards)| {
            let places = cards.iter().map(|card| mix(self.place_key(card))).fold(0, u64::wrapping_add);
            mix((key, places))
        });
        let script_state = self.script_state.borrow().iter().map(mix).fold(0, u64::wrapping_add);
        let located = self.player.0.iter().flat_map(|info| {
            let piles = [
                &info.list_main,
                &info.list_hand,
                &info.list_grave,
                &info.list_remove,
                &info.list_extra,
            ];
            piles
                .into_iter()
                .flatten()
                .chain(info.list_mzone.iter().chain(&info.list_szone).flatten())
        });
        let cards = (located.copied().chain(self.overlay_cards.iter()))
            .map(|card| {
                self.card_hash(card)
                    .wrapping_add(refs.as_ref().map_or(0, |refs| refs.card_flow(card)))
            })
            .fold(0, u64::wrapping_add);
        let effects = (self.effects.iter())
            .filter(|effect| (effect.card_key.is_some() || effect.field_key.is_some()) && !is_as_registered(effect))
            .map(|effect| self.effect_hash(effect));
        let parts = (count_codes.chain(once).chain(activities).chain(materials).chain(counters))
            .chain(targets.chain(relations).chain(spent).chain(script_groups).chain(effects))
            .fold(cards.wrapping_add(script_state), u64::wrapping_add);
        let flow = refs.map_or(0, |refs| refs.flow(prompt));
        Some(mix((turn, activity, &self.random)).wrapping_add(parts).wrapping_add(flow))
    }

    // The code and place of a card, without its order in the hand or the Extra Deck; a material is placed by its monster.
    fn place_key(&self, card: CardId) -> [u32; 5] {
        let place = self.cards.get_info_location(card);
        let sequence = match place.location & (LOCATION_HAND | LOCATION_EXTRA) {
            0 => place.sequence,
            _ => 0,
        };
        [
            self.cards[card].data.code,
            place.controler.into(),
            place.location.into(),
            sequence,
            place.position,
        ]
    }

    // The fields as one array of words, which the hasher reads 16 bytes at a time instead of one multiply per field.
    fn card_hash(&self, id: CardId) -> u64 {
        let card = &self.cards[id];
        let (current, summon, previous) = (&card.current, &card.summon, &card.previous);
        let ([spsummons, other_spsummons], [resets, other_resets]) = (card.spsummon_counter.0, card.spsummon_counter_rst.0);
        let [code, controler, location, sequence, position] = self.place_key(id);
        let words = [
            code,
            controler,
            location,
            sequence,
            position,
            card.owner.into(),
            current.pzone.into(),
            card.status,
            current.reason,
            current.reason_player.into(),
            card.turnid.into(),
            card.turn_counter.into(),
            summon.summon_type,
            summon.location.into(),
            summon.pzone.into(),
            summon.player.into(),
            previous.controler.into(),
            previous.location.into(),
            previous.position,
            card.attack_announce_count.into(),
            card.announce_count.into(),
            card.attacked_count.into(),
            card.attack_all_target.into(),
            card.direct_attackable.into(),
            spsummons.into(),
            other_spsummons.into(),
            resets.into(),
            other_resets.into(),
            card.spsummon_code,
        ];
        mix((words, card.equiping_target.map(|target| self.place_key(target))))
    }

    fn effect_hash(&self, effect: &Effect) -> u64 {
        let (is_int, value) = match effect.value {
            Value::Int(value) => (1, value as u32),
            _ => (0, 0),
        };
        let ([flag, flag2], [timing, other_timing]) = (effect.flag, effect.hint_timing);
        let words = [
            self.cards[effect.owner].data.code,
            effect.effect_owner.into(),
            effect.code,
            effect.effect_type.into(),
            flag,
            flag2,
            effect.range.into(),
            effect.s_range.into(),
            effect.o_range.into(),
            effect.card_type,
            effect.active_type,
            effect.active_location.into(),
            timing,
            other_timing,
            effect.reset_flag,
            effect.reset_count.into(),
            effect.count_code,
            effect.count_flag.into(),
            effect.count_hopt_index.into(),
            effect.count_limit.into(),
            effect.count_limit_max.into(),
            effect.label_object.is_some().into(),
            is_int,
            value,
        ];
        let handler = effect.handler.map(|handler| self.place_key(handler));
        mix((words, effect.description, effect.category, effect.labels(), handler))
    }
}

// What a state off the idle prompt refers to by id, keyed by content so that a key means the same card or effect in every
// order of actions, also when copies of a card differ in state: a card by its place and fields and its own effects, an
// effect by its content, functions and card. Each card's key is found once.
pub(crate) struct Refs<'a> {
    duel: &'a Duel,
    contents: [Cell<u64>; MAX_CARDS],
    effects: RefCell<List<(EffectId, u64), 32>>,
}

impl<'a> Refs<'a> {
    fn new(duel: &'a Duel) -> Self {
        Self {
            duel,
            contents: std::array::from_fn(|_| Cell::new(0)),
            effects: RefCell::default(),
        }
    }

    fn card(&self, card: CardId) -> u64 {
        let Some(slot) = self.contents.get(card.index()) else {
            return self.content(card);
        };
        if slot.get() == 0 {
            slot.set(self.content(card));
        }
        slot.get()
    }

    // Never 0, which marks a key not found yet.
    fn content(&self, card: CardId) -> u64 {
        let duel = self.duel;
        let effects = (duel.field_effects.registry.of_card(card))
            .map(|effect| &duel.effects[effect])
            .filter(|effect| !is_as_registered(effect))
            .map(|effect| duel.effect_hash(effect));
        let counters = (duel.counters.iter())
            .filter(|&&(other, ..)| other == card)
            .map(|&(_, kind, counts)| mix((kind, counts)));
        mix((duel.card_hash(card), self.words(card), summed(effects), summed(counters))) | 1
    }

    fn cards(&self, cards: CardSet) -> u64 {
        summed(cards.iter().map(|card| self.card(card)))
    }

    fn list(&self, cards: &[CardId]) -> u64 {
        chained(cards.iter().map(|&card| self.card(card)))
    }

    fn effect(&self, effect: EffectId) -> u64 {
        if let Some(&(_, key)) = self.effects.borrow().iter().find(|(other, _)| *other == effect) {
            return key;
        }
        let peffect = &self.duel.effects[effect];
        let functions = (
            peffect.condition,
            peffect.cost,
            peffect.cost_condition,
            peffect.target,
            peffect.value,
            peffect.operation,
        );
        let key = mix((
            self.duel.effect_hash(peffect),
            functions,
            peffect.handler.map(|card| self.card(card)),
        ));
        let mut effects = self.effects.borrow_mut();
        if effects.len() < 32 {
            effects.push((effect, key));
        }
        key
    }

    // An event id relative to the counter, which only raised events advance.
    fn event_age(&self, id: u32) -> u32 {
        self.duel.infos.event_id.wrapping_sub(id)
    }

    fn event(&self, event: &Event) -> u64 {
        let Event {
            trigger_card,
            event_cards,
            reason_effect,
            event_code,
            event_value,
            reason,
            event_player,
            reason_player,
            global_id,
        } = *event;
        let words = [
            event_code,
            event_value,
            reason,
            event_player.into(),
            reason_player.into(),
            self.event_age(global_id),
        ];
        let cards = (trigger_card.map(|card| self.card(card)), event_cards.map(|cards| self.cards(cards)));
        mix((words, cards, reason_effect.map(|effect| self.effect(effect))))
    }

    fn continuous(&self, continuous: &Continuous) -> u64 {
        mix((self.effect(continuous.effect), continuous.player, self.event(&continuous.event)))
    }

    // A chain link without its id, which the relations refer to by the link's place instead.
    fn chain(&self, chain: &Chain) -> u64 {
        let Chain {
            triggering_state,
            evt,
            chain_count,
            triggering_player,
            triggering_controler,
            triggering_position,
            target_player,
            disable_player,
            triggering_summon_location,
            triggering_summon_proc_complete,
            was_just_sent,
            chain_id: _,
            triggering_location,
            triggering_sequence,
            triggering_status,
            triggering_summon_type,
            replace_op,
            target_param,
            flag,
            event_id,
            triggering_effect,
            applied_chain_counters,
            target_cards,
            disable_reason,
            opinfos,
            possibleopinfos,
        } = chain;
        let words = [
            (*chain_count).into(),
            (*triggering_player).into(),
            (*triggering_controler).into(),
            (*triggering_position).into(),
            (*target_player).into(),
            (*disable_player).into(),
            (*triggering_summon_location).into(),
            (*triggering_summon_proc_complete).into(),
            (*was_just_sent).into(),
            (*triggering_location).into(),
            field_sequence((*triggering_location).into(), *triggering_sequence),
            *triggering_status,
            *triggering_summon_type,
            *target_param as u32,
            *flag,
            self.event_age(*event_id),
            (*applied_chain_counters).into(),
        ];
        let infos = |infos: &OpInfos| {
            summed(infos.0.iter().flatten().map(|(category, target)| {
                let cards = target.op_cards.map(|cards| self.cards(cards));
                mix((category, cards, target.op_count, target.op_player, target.op_param))
            }))
        };
        let effects = (self.effect(*triggering_effect), disable_reason.map(|effect| self.effect(effect)));
        let targets = target_cards.map(|cards| self.cards(cards));
        mix((
            words,
            triggering_state,
            self.event(evt),
            replace_op,
            effects,
            targets,
            infos(opinfos),
            infos(possibleopinfos),
        ))
    }

    // A card's fields that a move or an operation in progress leaves for events and units to read; not the reasons in
    // `previous` and `temp.reason_card`, which nothing reads, nor the controller and location of `temp`, which nothing writes.
    fn words(&self, id: CardId) -> ([u32; 20], [u8; 4]) {
        let card = &self.duel.cards[id];
        let (previous, temp, send) = (&card.previous, &card.temp, card.sendto_param);
        let words = [
            card.previous_type,
            card.previous_attribute,
            card.previous_race as u32,
            (card.previous_race >> 32) as u32,
            card.previous_attack as u32,
            card.previous_defense as u32,
            card.previous_code,
            card.previous_code2,
            field_sequence(previous.location.into(), previous.sequence),
            previous.pzone.into(),
            temp.sequence,
            temp.position,
            temp.reason,
            temp.reason_player.into(),
            card.to_field_param,
            card.position_param,
            card.release_param,
            card.spsummon_param,
            card.attack_controler.into(),
            field_sequence(card.summon.location.into(), card.summon.sequence),
        ];
        (words, [send.playerid, send.position, send.location, send.sequence])
    }

    // [Refs::words] with the cards and effects the card's fields refer to.
    fn card_flow(&self, id: CardId) -> u64 {
        let card = &self.duel.cards[id];
        let (current, temp) = (&card.current, &card.temp);
        let (words, send) = self.words(id);
        // `temp` keeps the reason saved around an operation, mostly the current one.
        let saved = (temp.reason_effect != current.reason_effect).then_some(temp.reason_effect);
        let cards = [current.reason_card, card.pre_overlay_target, card.pre_equip_target];
        // A card no operation has touched since add_card set its send position is alike in every order.
        let [playerid, _, location, sequence] = send;
        let untouched = words == [0; 20] && (playerid, location, sequence) == (0, 0, 0);
        if untouched && current.reason_effect.is_none() && saved.is_none() && cards == [None; 3] && card.unique.is_none() {
            return 0;
        }
        let reference = |effect: Option<EffectId>| effect.map(|effect| self.effect(effect));
        let reasons = (reference(current.reason_effect), saved.map(reference));
        let unique = card
            .unique
            .map(|unique| (unique.filter, unique.pos, unique.location, self.effect(unique.effect)));
        mix((words, send, reasons, cards.map(|card| card.map(|card| self.card(card))), unique))
    }

    // The battle records the attackable cards are found from, a target by the card now holding its field id.
    fn battles(&self) -> u64 {
        let duel = self.duel;
        if duel.announced_cards.is_empty() && duel.attacked_cards.is_empty() && duel.battled_cards.is_empty() {
            return 0;
        }
        let located = duel.located_cards();
        // None for a direct attack, Some(None) for a card that has moved since.
        let target = |field_id: u32| {
            let holder = located.iter().find(|&&card| duel.cards[card].fieldid_r == field_id);
            (field_id != 0).then(|| holder.map(|&card| self.card(card)))
        };
        let announced = (duel.announced_cards.iter()).map(|&(card, field_id, count)| mix((0u8, self.card(card), target(field_id), count)));
        let attacked = (duel.attacked_cards.iter()).map(|&(card, field_id)| mix((1u8, self.card(card), target(field_id))));
        let battled = (duel.battled_cards.iter()).map(|&(card, field_id)| mix((2u8, self.card(card), target(field_id))));
        summed(announced.chain(attacked).chain(battled))
    }

    // The processor's state beyond the board's part, each field named so that a new one fails to compile until it is placed.
    fn flow(&self, prompt: Prompt) -> u64 {
        let duel = self.duel;
        let Processor {
            units,
            subunits,
            recover_damage_reserve,
            just_sent_cards,
            select_cards,
            unselect_cards,
            select_options,
            coin_results,
            dice_results,
            select_effects,
            cards_to_summon_g,
            release_cards,
            release_cards_ex,
            release_cards_ex_oneof,
            point_event,
            instant_event,
            queue_event,
            delayed_activate_event,
            full_event,
            single_event,
            solving_event,
            sub_solving_event,
            select_chains,
            tpchain,
            current_chain,
            real_chain_count,
            ignition_priority_chains,
            continuous_chain,
            readjust_map,
            desrep_chain,
            replace_targets,
            solving_continuous,
            sub_solving_continuous,
            delayed_continuous_tp,
            delayed_continuous_ntp,
            continuous_tp,
            continuous_ntp,
            new_fchain,
            new_fchain_s,
            new_ochain,
            new_ochain_s,
            new_fchain_b,
            new_ochain_b,
            new_ochain_h,
            new_chains,
            delayed_quick_tmp,
            delayed_quick,
            quick_f_chain,
            chain_limit,
            chain_limit_p,
            send_to_args,
            leave_confirmed,
            special_summoning,
            equiping_cards,
            ss_tograve_set,
            delayed_enable_set,
            destroy_canceled,
            battle_destroy_rep,
            battle_destroyed,
            operated_set,
            reseted_effects,
            chain_solving,
            conti_solving,
            conti_player,
            check_level,
            win_player,
            win_reason,
            re_adjust,
            reason_effect,
            reason_player,
            attacker,
            attack_target,
            pre_field,
            opp_mzone,
            attack_rollback,
            battle_damage,
            skip_m2,
            chain_attack,
            chain_attacker_id,
            chain_attack_target,
            attack_player,
            selfdes_disabled,
            unique_destroy_set,
            self_destroy_set,
            overdraw,
            shuffle_check_disabled,
            shuffle_hand_check,
            trap_monster_adjust_set,
            shuffle_deck_check,
            deck_reversed,
            flip_delayed,
            damage_calculated,
            hand_adjusted,
            spsummon_state_count_tmp,
            phase_action,
            hint_timing,
            current_player,
            force_turn_end,
            attack_cancelable,
            attack_cost_paid,
            summon_depth,
            summon_cancelable,
            reserved,
            effect_damage_step,
            summoning_card,
            spsummon_group,
            fusion_materials,
            copy_reset,
            copy_reset_count,
            must_select_cards,
            // In the board's part.
            custom_counters: _,
            effect_count_code: _,
            effect_count_code_duel: _,
            effect_count_code_chain: _,
            global_flag: _,
            summon_count: _,
            extra_summon: _,
            spe_effect: _,
            summon_state_count: _,
            normalsummon_state_count: _,
            flipsummon_state_count: _,
            spsummon_state_count: _,
            spsummon_state_count_rst: _,
            spsummon_once_map: _,
            spsummon_rst: _,
            attack_state_count: _,
            battle_phase_count: _,
            battled_count: _,
            duel_options: _,
            // Filled from the state when an idle or battle prompt opens, and read only there.
            summonable_cards: _,
            spsummonable_cards: _,
            repositionable_cards: _,
            msetable_cards: _,
            ssetable_cards: _,
            attackable_cards: _,
            to_bp: _,
            to_m2: _,
            to_ep: _,
            // Set only while a stat or type is worked out.
            stat_temp: _,
            base_stat_temp: _,
            type_temp: _,
        } = &duel.core;
        let sets = [
            just_sent_cards,
            cards_to_summon_g,
            release_cards,
            release_cards_ex,
            release_cards_ex_oneof,
            leave_confirmed,
            special_summoning,
            equiping_cards,
            ss_tograve_set,
            delayed_enable_set,
            destroy_canceled,
            battle_destroy_rep,
            battle_destroyed,
            operated_set,
            unique_destroy_set,
            self_destroy_set,
            spsummon_group,
            fusion_materials,
            &duel.field_effects.disable_check_set,
        ];
        let sets = (sets.into_iter().copied())
            .chain(trap_monster_adjust_set.0)
            .chain(replace_targets.iter().copied())
            .chain([must_select_cards.get()]);
        let sets = chained(sets.map(|cards| self.cards(cards)));
        let lists: [&[CardId]; 3] = [select_cards, unselect_cards, &duel.return_cards.list];
        let lists = chained(lists.map(|cards| self.list(cards)));
        let events: [&[Event]; 9] = [
            point_event,
            instant_event,
            queue_event,
            delayed_activate_event,
            full_event,
            single_event,
            solving_event,
            sub_solving_event,
            &[duel.nil_event],
        ];
        let events = chained(events.map(|events| chained(events.iter().map(|event| self.event(event)))));
        let delayed = [delayed_quick_tmp, delayed_quick]
            .map(|quick| chained(quick.iter().map(|(effect, event)| mix((self.effect(*effect), self.event(event))))));
        let chain_lists: [&[Chain]; 13] = [
            current_chain,
            new_chains,
            select_chains,
            ignition_priority_chains,
            continuous_chain,
            desrep_chain,
            new_fchain,
            new_fchain_s,
            new_ochain,
            new_ochain_s,
            new_fchain_b,
            new_ochain_b,
            new_ochain_h,
        ];
        let quick = chained(
            quick_f_chain
                .iter()
                .map(|(effect, chain)| mix((self.effect(*effect), self.chain(chain)))),
        );
        let chains = chained(chain_lists.map(|chains| chained(chains.iter().map(|chain| self.chain(chain)))));
        // A chain id by where its link is, as ids come from the field id counter.
        let chain_place = |id: u16| {
            let quick = quick_f_chain.iter().map(|(_, chain)| chain);
            let mut links = chain_lists.iter().flat_map(|chains| chains.iter()).chain(quick);
            links.position(|chain| chain.chain_id == id).ok_or(id)
        };
        let relations = (duel.relations.iter()).map(|relation| {
            mix((
                self.card(relation.card),
                self.effect(relation.effect),
                chain_place(relation.chain_id),
            ))
        });
        let oaths =
            (duel.field_effects.oath.iter()).map(|&(effect, reason)| mix((self.effect(effect), reason.map(|reason| self.effect(reason)))));
        let continuous = [
            solving_continuous,
            sub_solving_continuous,
            delayed_continuous_tp,
            delayed_continuous_ntp,
            continuous_tp,
            continuous_ntp,
        ];
        let continuous = continuous.map(|list| chained(list.iter().map(|continuous| self.continuous(continuous))));
        let limits = [chain_limit, chain_limit_p]
            .map(|limits| chained(limits.iter().map(|limit| mix((limit.function, limit.capture, limit.player)))));
        let send_to = send_to_args.iter().map(|args| {
            let sets = [args.leave_field, args.leave_grave, args.detach].map(|cards| self.cards(cards));
            mix((sets, self.list(&args.cv), args.cvit, args.check_decktop_visibility))
        });
        let readjust = summed(readjust_map.iter().map(|(&card, count)| mix((self.card(card), count))));
        let words = [
            *real_chain_count as u32,
            (*chain_solving).into(),
            (*conti_solving).into(),
            (*conti_player).into(),
            *check_level as u32,
            (*win_player).into(),
            (*win_reason).into(),
            (*re_adjust).into(),
            reason_player.get().into(),
            pre_field[0],
            pre_field[1],
            (*attack_rollback).into(),
            battle_damage.0[0] as u32,
            battle_damage.0[1] as u32,
            (*skip_m2).into(),
            (*chain_attack).into(),
            *chain_attacker_id,
            (*attack_player).into(),
            (*selfdes_disabled).into(),
            (*shuffle_check_disabled).into(),
            (*deck_reversed).into(),
            (*flip_delayed).into(),
            (*damage_calculated).into(),
            (*hand_adjusted).into(),
            (*phase_action).into(),
            hint_timing.0[0],
            hint_timing.0[1],
            (*current_player).into(),
            (*force_turn_end).into(),
            (*attack_cancelable).into(),
            (*attack_cost_paid).into(),
            (*summon_depth).into(),
            (*summon_cancelable).into(),
            (*effect_damage_step).into(),
            *copy_reset,
            (*copy_reset_count).into(),
        ];
        let players = (
            overdraw,
            shuffle_hand_check,
            shuffle_deck_check,
            spsummon_state_count_tmp,
            duel.infos.priorities,
        );
        let cards = [attacker, attack_target, chain_attack_target, summoning_card].map(|card| card.map(|card| self.card(card)));
        let effects = |effects: &[EffectId]| chained(effects.iter().map(|&effect| self.effect(effect)));
        let effects = (
            effects(select_effects),
            effects(tpchain),
            effects(reseted_effects),
            reason_effect.get().map(|effect| self.effect(effect)),
        );
        // The picks so far of a prompt answered a pick at a time; any other answer overwrites the buffer whole.
        let picks = match prompt {
            Prompt::Card { .. }
            | Prompt::Tribute { .. }
            | Prompt::Sum { .. }
            | Prompt::Place { .. }
            | Prompt::AnnounceRace { .. }
            | Prompt::AnnounceAttribute { .. }
            | Prompt::Sort { .. }
            | Prompt::Counter { .. } => Some(&*duel.returns.data),
            _ => None,
        };
        let fusion = duel.fusion_check_additional.map(|(check, card)| (check, self.card(card)));
        let assumes = summed(
            duel.assumes
                .borrow()
                .iter()
                .map(|&(card, kind, value)| mix((self.card(card), kind, value))),
        );
        let scripts = (fusion, duel.synchro_check_additional, duel.ritual_summoning_level.get(), assumes);
        let lists_of_values = (
            &**select_options,
            &**coin_results,
            &**dice_results,
            &**opp_mzone,
            duel.return_cards.canceled,
        );
        let mut state = FxHasher::default();
        (units, subunits, recover_damage_reserve, reserved).key(self, &mut state);
        let units = state.finish();
        let processor = (words, players, cards, effects, picks, scripts, lists_of_values);
        let pending = (sets, lists, events, (chains, quick, delayed), continuous, limits, readjust);
        mix((units, processor, pending))
            .wrapping_add(chained(send_to))
            .wrapping_add(summed(relations))
            .wrapping_add(summed(oaths))
            .wrapping_add(self.battles())
            .wrapping_add(self.links())
    }

    // The board's links between cards again, by key, which tells copies apart where the board's places do not.
    fn links(&self) -> u64 {
        let duel = self.duel;
        let targets = (duel.card_targets.iter()).map(|&(card, target)| mix((0u8, self.card(card), self.card(target))));
        let relations = (duel.card_relations.iter()).map(|&(card, other, reset)| mix((1u8, self.card(card), self.card(other), reset)));
        let materials = (duel.material_cards.iter()).map(|&(card, materials)| mix((2u8, self.card(card), self.cards(materials))));
        let groups = (duel.script_groups.iter()).map(|&(key, cards)| mix((3u8, key, self.cards(cards))));
        summed(targets.chain(relations).chain(materials).chain(groups))
    }
}

// A unit's fields hashed with the cards and effects they hold by [Refs]' keys.
pub(crate) trait Keyed {
    fn key(&self, refs: &Refs, state: &mut FxHasher);
}

macro_rules! keyed_by_hash {
    ($($type:ty),*) => {$(
        impl Keyed for $type {
            fn key(&self, _: &Refs, state: &mut FxHasher) {
                self.hash(state);
            }
        }
    )*};
}

keyed_by_hash!(bool, u8, u16, u32, u64, i32, usize, Operation);

impl Keyed for CardId {
    fn key(&self, refs: &Refs, state: &mut FxHasher) {
        state.write_u64(refs.card(*self));
    }
}

impl Keyed for EffectId {
    fn key(&self, refs: &Refs, state: &mut FxHasher) {
        state.write_u64(refs.effect(*self));
    }
}

impl Keyed for CardSet {
    fn key(&self, refs: &Refs, state: &mut FxHasher) {
        state.write_u64(refs.cards(*self));
    }
}

impl Keyed for ExecuteKind {
    fn key(&self, _: &Refs, state: &mut FxHasher) {
        state.write_u8(*self as u8);
    }
}

impl<T: Keyed> Keyed for Option<T> {
    fn key(&self, refs: &Refs, state: &mut FxHasher) {
        state.write_u8(self.is_some().into());
        if let Some(value) = self {
            value.key(refs, state);
        }
    }
}

impl<T: Keyed + ?Sized> Keyed for Box<T> {
    fn key(&self, refs: &Refs, state: &mut FxHasher) {
        (**self).key(refs, state);
    }
}

impl<T: Keyed> Keyed for [T] {
    fn key(&self, refs: &Refs, state: &mut FxHasher) {
        state.write_usize(self.len());
        self.iter().for_each(|item| item.key(refs, state));
    }
}

impl<T: Keyed> Keyed for Vec<T> {
    fn key(&self, refs: &Refs, state: &mut FxHasher) {
        self.as_slice().key(refs, state);
    }
}

impl<T: Keyed, const N: usize> Keyed for [T; N] {
    fn key(&self, refs: &Refs, state: &mut FxHasher) {
        self.as_slice().key(refs, state);
    }
}

impl<A: Keyed, B: Keyed> Keyed for (A, B) {
    fn key(&self, refs: &Refs, state: &mut FxHasher) {
        self.0.key(refs, state);
        self.1.key(refs, state);
    }
}

impl<A: Keyed, B: Keyed, C: Keyed, D: Keyed> Keyed for (A, B, C, D) {
    fn key(&self, refs: &Refs, state: &mut FxHasher) {
        self.0.key(refs, state);
        self.1.key(refs, state);
        self.2.key(refs, state);
        self.3.key(refs, state);
    }
}

impl<T: Keyed + ?Sized> Keyed for &T {
    fn key(&self, refs: &Refs, state: &mut FxHasher) {
        (**self).key(refs, state);
    }
}

// Every field named, so that a new one fails to compile until it is listed.
macro_rules! keyed {
    ($($type:ty { $($field:ident),* })*) => {$(
        impl Keyed for $type {
            fn key(&self, refs: &Refs, state: &mut FxHasher) {
                let Self { $($field),* } = self;
                $($field.key(refs, state);)*
            }
        }
    )*};
}

keyed! {
    processor::Adjust { step }
    processor::Turn { step, turn_player, has_performed_second_battle_phase }
    processor::RefreshLoc { step, previously_disabled_locations, disfield_effects, current_disable_field_effect }
    processor::Startup { step, turn }
    processor::SelectBattleCmd { step, playerid }
    processor::SelectIdleCmd { step, playerid }
    processor::SelectEffectYesNo { step, playerid, description, card }
    processor::SelectYesNo { step, playerid, description }
    processor::SelectOption { step, playerid }
    processor::AnnounceNumber { step, playerid }
    processor::AnnounceRace { step, playerid, count, available }
    processor::AnnounceAttribute { step, playerid, count, available }
    processor::AnnounceCard { step, playerid }
    processor::SortCard { step, playerid, is_chain }
    processor::SortChain { step, playerid }
    processor::SortDeck { step, sort_player, target_player, count, bottom }
    processor::SelectCard { step, playerid, cancelable, min, max }
    processor::SelectUnselectCard { step, playerid, cancelable, min, max, finishable }
    processor::SelectCounter { step, playerid, counter_type, count, capped, self_side, oppo }
    processor::SelectSum { step, playerid, acc, min, max, params }
    processor::SelectChain { step, playerid, spe_count, forced }
    processor::SelectPlace { step, playerid, count, flag, disable_field, card }
    processor::SelectPosition { step, playerid, positions, card }
    processor::SelectTributeP { step, playerid, cancelable, min, max }
    operations::SelectTribute { step, min, max, playerid, cancelable, toplayer, zone, target, extra_release_effect, must_choose_one }
    processor::QuickEffect { step, skip_freechain, is_opponent, priority_player }
    processor::IdleCommand { step, phase_to_change_to, card_to_reposition }
    processor::PhaseEvent { step, phase, is_opponent, priority_passed }
    processor::PointEvent { step, skip_trigger, skip_freechain, skip_new }
    processor::BattleCommand { step, phase_to_change_to, forced_attack_done, is_replaying_attack, attack_announce_failed, repeat_battle_phase, second_battle_phase_is_optional, previous_point_event_had_any_trigger_to_resolve, reason_player, cards_destroyed_by_battle, reason_card }
    processor::DamageStep { step, attacker, attack_target, new_attack, backup_phase, cards_destroyed_by_battle }
    operations::Destroy { step, reason_player, reason, targets, reason_effect, replacing }
    operations::DestroyReplace { step, battle, target }
    operations::OperationReplace { step, replace_effect, target, is_destroy }
    operations::Release { step, reason_player, reason, targets, reason_effect }
    operations::SendTo { step, reason_player, reason, targets, reason_effect }
    operations::MoveToField { step, enable, pzone, zone, rule, location_reason, confirm, ret, target }
    operations::ChangePos { step, reason_player, enable, oppo_selection, reason_effect, targets, to_grave_set }
    operations::SummonRule { step, sumplayer, min_tribute, ignore_count, zone, target, summon_procedure_effect, extra_summon_effect, tributes, ordinary_first, ordinary_count_first }
    operations::FlipSummon { step, sumplayer, target }
    operations::MonsterSet { step, setplayer, min_tribute, ignore_count, zone, target, summon_procedure_effect, extra_summon_effect, tributes, ordinary_first }
    operations::SpellSet { step, setplayer, toplayer, target, reason_effect }
    operations::SpSummon { step, reason_player, zone, reason_effect, targets }
    operations::SpSummonStep { step, zone, in_group, target, spsummon_cost_effects }
    operations::SpSummonRule { step, sumplayer, target, summon_type, summon_proc_effect, proc_g, procedure, spsummon_cost_effects }
    operations::SpellSetGroup { step, setplayer, toplayer, confirm, reason_effect, state }
    operations::RemoveCounter { step, reason, card, rplayer, self_side, oppo, counter_type, count }
    operations::RemoveOverlay { step, reason, group, rplayer, self_side, oppo, min, max, has_used_overlay_remove_replace_effect, replaced_amount }
    operations::XyzOverlay { step, send_materials_to_grave, target, materials }
    operations::Draw { step, count, reason_player, playerid, reason, reason_effect, drawn_set }
    operations::DiscardDeck { step, playerid, count, reason, discarded_set }
    operations::Equip { step, equip_player, equip_card, target, faceup, is_step }
    operations::GetControl { step, reason_effect, chose_player, targets, playerid, reset_phase, reset_count, zone, destroy_set, remaining }
    operations::SwapControl { step, reason_effect, reason_player, targets1, targets2, reset_phase, reset_count, index, self_selected_sequence }
    operations::TrapMonsterAdjust { step, oppo_selection, to_grave_set }
    operations::SelfDestroyUnique { step, unique_card, playerid }
    operations::SelfDestroy { step }
    operations::TossCoin { step, reason_effect, reason_player, playerid, count }
    operations::TossDice { step, reason_effect, reason_player, playerid, count1, count2 }
    operations::Damage { step, reason_player, playerid, amount, reason, reason_card, reason_effect, is_reflected, is_step }
    operations::Recover { step, reason_player, playerid, amount, reason, reason_effect }
    operations::PayLpCost { step, playerid, cost }
    chain::Execute { step, kind, triggering_effect, triggering_player, procedure }
    chain::AddChain { step, is_activated_effect }
    chain::SolveChain { step, skip_trigger, skip_freechain, skip_new, backed_up_operation }
    chain::SolveContinuous { step, reason_player, reason_effect }
    processor::RefreshRelay { step }
    script::Procedure { card, must, materials, min, max, chkf, zone, releasable }
    operations::ProcGroup { cards, remaining }
    operations::SpellSetGroupState { targets, set_cards, pre_set, set, used_zones, seq }
}
