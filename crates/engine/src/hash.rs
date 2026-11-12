use std::hash::{BuildHasher, Hash};

use rustc_hash::FxBuildHasher;

use crate::action::Prompt;
use crate::card::CardId;
use crate::constants::*;
use crate::duel::Duel;
use crate::effect::{Effect, Value};

// Fx with a murmur3 finish, so that parts summed in any order still spread over all bits.
fn mix(part: impl Hash) -> u64 {
    let mut hash = FxBuildHasher.hash_one(part);
    hash = (hash ^ hash >> 33).wrapping_mul(0xff51_afd7_ed55_8ccd);
    hash = (hash ^ hash >> 33).wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    hash ^ hash >> 33
}

// An initial effect as the card's script registered it, which the card's code already stands for.
fn is_as_registered(effect: &Effect) -> bool {
    effect.is_flag(EFFECT_FLAG_INITIAL)
        && effect.count_limit == effect.count_limit_max
        && effect.label_len == 0
        && effect.label_object.is_none()
}

impl Duel {
    // The same for the same state reached by another order of actions, as it hashes cards and effects by content, never
    // by an id, field id or timestamp; Some only at an idle prompt, where no function waits. What it leaves out (function
    // pointers, effect statuses, initial effects as registered, battle records) can merge distinct states.
    pub fn state_hash(&self) -> Option<u64> {
        if !matches!(self.prompt(), Some(Prompt::IdleCommand { .. })) || self.replay.snapshot.is_some() {
            return None;
        }
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
            .map(|card| self.card_hash(card))
            .fold(0, u64::wrapping_add);
        let effects = (self.effects.iter())
            .filter(|effect| (effect.card_key.is_some() || effect.field_key.is_some()) && !is_as_registered(effect))
            .map(|effect| self.effect_hash(effect));
        let parts = (count_codes.chain(once).chain(activities).chain(materials).chain(counters))
            .chain(targets.chain(relations).chain(spent).chain(script_groups).chain(effects))
            .fold(cards.wrapping_add(script_state), u64::wrapping_add);
        Some(mix((turn, activity, &self.random)).wrapping_add(parts))
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
