// Port of libcard.cpp: the Card methods on CardId. Methods a library script replaces carry a `_core` suffix and the
// replacement takes the Lua name in the scripts crate.
use crate::card::{CardId, CardSet};
use crate::constants::*;
use crate::duel::Duel;
use crate::effect::{Container, EffectId, EffectSet};
use crate::script::Script;

impl CardId {
    pub fn register_effect(self, duel: &mut Duel, effect: EffectId) -> i32 {
        self.register_effect_full(duel, effect, false)
    }

    pub fn register_effect_full(self, duel: &mut Duel, effect: EffectId, forced: bool) -> i32 {
        if duel.effects[effect].owner == CardId::default() {
            return 0;
        }
        if !forced
            && let Some(reason_effect) = duel.core.reason_effect.get()
            && !duel.is_affect_by_effect(self, Some(reason_effect))
        {
            if !duel.core.reseted_effects.contains(&effect) {
                duel.core.reseted_effects.push(effect);
            }
            return 0;
        }
        if duel.effects[effect].handler.is_some() {
            return -1;
        }
        duel.card_add_effect(self, effect) as i32
    }

    pub fn register_flag_effect(self, duel: &mut Duel, code: u32, reset: u32, flag: u32, count: u16) -> EffectId {
        self.register_flag_effect_full(duel, code, reset, flag, count, 0, 0)
    }

    pub fn register_flag_effect_full(
        self,
        duel: &mut Duel,
        code: u32,
        reset: u32,
        flag: u32,
        count: u16,
        label: i64,
        description: u64,
    ) -> EffectId {
        let mut reset = reset;
        if reset & RESET_PHASE != 0 && reset & (RESET_SELF_TURN | RESET_OPPO_TURN) == 0 {
            reset |= RESET_SELF_TURN | RESET_OPPO_TURN;
        }
        let effect = duel.new_effect(self);
        let peffect = &mut duel.effects[effect];
        peffect.effect_type = EFFECT_TYPE_SINGLE;
        peffect.code = (code & 0xfff_ffff) | 0x1000_0000;
        peffect.reset_flag = reset;
        peffect.flag[0] = flag | EFFECT_FLAG_CANNOT_DISABLE;
        peffect.reset_count = count.max(1);
        peffect.set_labels(&[label]);
        peffect.description = description;
        duel.card_add_effect(self, effect);
        effect
    }

    pub fn is_can_be_special_summoned(
        self,
        duel: &Duel,
        effect: EffectId,
        sumtype: u32,
        sumplayer: u8,
        nocheck: bool,
        nolimit: bool,
    ) -> bool {
        self.is_can_be_special_summoned_full(duel, effect, sumtype, sumplayer, nocheck, nolimit, POS_FACEUP, sumplayer, 0xff)
    }

    pub fn is_can_be_special_summoned_full(
        self,
        duel: &Duel,
        effect: EffectId,
        sumtype: u32,
        sumplayer: u8,
        nocheck: bool,
        nolimit: bool,
        position: u32,
        toplayer: u8,
        zone: u32,
    ) -> bool {
        duel.is_can_be_special_summoned(self, effect, sumtype, position as u8, sumplayer, toplayer, nocheck, nolimit, zone)
    }

    pub fn is_location(self, duel: &Duel, location: u32) -> bool {
        let card = &duel.cards[self];
        let location = location as u16;
        match card.current.location {
            LOCATION_MZONE => {
                card.current.is_location(location) && !card.get_status(STATUS_SUMMONING | STATUS_SUMMON_DISABLED | STATUS_SPSUMMON_STEP)
            }
            LOCATION_SZONE => card.current.is_location(location) && !card.is_status(STATUS_ACTIVATE_DISABLED),
            other => u16::from(other) & location != 0,
        }
    }

    pub fn is_on_field(self, duel: &Duel) -> bool {
        let card = &duel.cards[self];
        card.current.location & LOCATION_ONFIELD != 0
            && !card.get_status(STATUS_SUMMONING | STATUS_SUMMON_DISABLED | STATUS_ACTIVATE_DISABLED | STATUS_SPSUMMON_STEP)
    }

    pub fn is_faceup(self, duel: &Duel) -> bool {
        duel.cards[self].is_position(POS_FACEUP)
    }

    pub fn is_facedown(self, duel: &Duel) -> bool {
        duel.cards[self].is_position(POS_FACEDOWN)
    }

    pub fn is_attack_pos(self, duel: &Duel) -> bool {
        duel.cards[self].is_position(POS_ATTACK)
    }

    pub fn is_defense_pos(self, duel: &Duel) -> bool {
        duel.cards[self].is_position(POS_DEFENSE)
    }

    pub fn get_attack(self, duel: &Duel) -> i32 {
        duel.get_attack(self).max(0)
    }

    pub fn is_attack_below(self, duel: &Duel, attack: i32) -> bool {
        if duel.cards[self].data.card_type & TYPE_MONSTER == 0
            && duel.get_type(self) & TYPE_MONSTER == 0
            && duel.is_affected_by_effect(self, EFFECT_PRE_MONSTER).is_none()
        {
            return false;
        }
        let current = duel.get_attack(self);
        current >= 0 && current <= attack
    }

    pub fn is_defense_below(self, duel: &Duel, defense: i32) -> bool {
        let card = &duel.cards[self];
        if card.data.card_type & TYPE_LINK != 0
            || (card.data.card_type & TYPE_MONSTER == 0
                && duel.get_type(self) & TYPE_MONSTER == 0
                && card.current.location & LOCATION_MZONE == 0)
        {
            return false;
        }
        let current = duel.get_defense(self);
        current >= 0 && current <= defense
    }

    pub fn is_can_be_effect_target(self, duel: &Duel, effect: EffectId) -> bool {
        duel.is_capable_be_effect_target(self, effect, duel.core.reason_player.get())
    }

    pub fn is_previous_location(self, duel: &Duel, location: u32) -> bool {
        duel.cards[self].previous.is_location(location as u16)
    }

    // card::is_capable_change_position_by_effect.
    pub fn is_can_change_position(self, duel: &Duel) -> bool {
        let card_type = duel.cards[self].data.card_type;
        !(card_type & TYPE_LINK != 0 && card_type & TYPE_MONSTER != 0)
    }

    pub fn is_can_turn_set(self, duel: &Duel) -> bool {
        duel.is_capable_turn_set(self, duel.core.reason_player.get())
    }

    pub fn get_previous_location(self, duel: &Duel) -> u32 {
        u32::from(duel.cards[self].previous.location)
    }

    // Card.IsPreviousSetCard: the archetypes of the card's previous code and set code effects, which come with the
    // cards that change a code or add an archetype; until then they are the archetypes it has.
    pub fn is_previous_set_card(self, duel: &Duel, setcodes: &[u16]) -> bool {
        self.is_set_card(duel, setcodes)
    }

    pub fn get_previous_sequence(self, duel: &Duel) -> u32 {
        duel.cards[self].previous.sequence
    }

    pub fn is_previous_position(self, duel: &Duel, position: u32) -> bool {
        duel.cards[self].previous.position & (position & 0xff) != 0
    }

    pub fn is_able_to_hand_core(self, duel: &Duel) -> bool {
        duel.is_capable_send_to_hand(self, duel.core.reason_player.get())
    }

    pub fn is_code(self, duel: &Duel, code: u32) -> bool {
        let another = duel.get_another_code(self);
        duel.get_code(self) == code || (another != 0 && another == code)
    }

    pub fn get_code(self, duel: &Duel) -> u32 {
        duel.get_code(self)
    }

    pub fn is_type(self, duel: &Duel, card_type: u32) -> bool {
        self.is_type_full(duel, card_type, None, 0, PLAYER_NONE)
    }

    // Card.IsType(c, type, scard, sumtype, playerid): the type as material of `scard` when one is given.
    pub fn is_type_full(self, duel: &Duel, card_type: u32, scard: Option<CardId>, sumtype: u64, player: u8) -> bool {
        duel.get_type_for(self, scard, sumtype, player) & card_type != 0
    }

    pub fn is_disabled(self, duel: &Duel) -> bool {
        duel.cards[self].is_status(STATUS_DISABLED)
    }

    pub fn is_status(self, duel: &Duel, status: u32) -> bool {
        duel.cards[self].get_status(status)
    }

    pub fn set_status(self, duel: &mut Duel, status: u32, enabled: bool) {
        let card = &mut duel.cards[self];
        if !card.is_status(STATUS_COPYING_EFFECT) {
            card.set_status(status, enabled);
        }
    }

    pub fn get_owner(self, duel: &Duel) -> u8 {
        duel.cards[self].owner
    }

    pub fn get_controler(self, duel: &Duel) -> u8 {
        duel.cards[self].current.controler
    }

    // The core indexes past an empty chain; here that is no relation.
    pub fn is_relate_to_chain(self, duel: &Duel, chain_count: u8) -> bool {
        let chains = &duel.core.current_chain;
        let index = match usize::from(chain_count) {
            0 => chains.len(),
            count if count > chains.len() => chains.len(),
            count => count,
        };
        index
            .checked_sub(1)
            .and_then(|index| chains.get(index))
            .is_some_and(|chain| duel.is_has_relation(self, chain.triggering_effect, chain.chain_id))
    }

    pub fn is_relate_to_effect_core(self, duel: &Duel, effect: EffectId) -> bool {
        duel.is_has_relation_effect(self, effect)
    }

    pub fn is_discardable(self, duel: &Duel) -> bool {
        self.is_discardable_full(duel, REASON_COST, None)
    }

    // `player` None is the reason player.
    pub fn is_discardable_full(self, duel: &Duel, reason: u32, player: Option<u8>) -> bool {
        let player = player.unwrap_or_else(|| duel.core.reason_player.get());
        (reason != REASON_COST || duel.is_affected_by_effect(self, EFFECT_CANNOT_USE_AS_COST).is_none())
            && duel.is_player_can_discard_hand(player, self)
    }
}

impl CardId {
    pub fn get_sequence(self, duel: &Duel) -> u32 {
        duel.cards[self].current.sequence
    }

    pub fn is_controler(self, duel: &Duel, player: u8) -> bool {
        duel.cards[self].current.controler == player
    }

    pub fn get_level(self, duel: &Duel) -> u32 {
        duel.get_level(self)
    }

    // Lua passes the levels to match as further arguments.
    pub fn is_level(self, duel: &Duel, levels: &[u32]) -> bool {
        levels.contains(&duel.get_level(self))
    }

    pub fn is_level_below(self, duel: &Duel, level: u32) -> bool {
        let current = duel.get_level(self);
        current > 0 && current <= level
    }

    pub fn get_original_level(self, duel: &Duel) -> u32 {
        let card = &duel.cards[self];
        if card.data.card_type & (TYPE_XYZ | TYPE_LINK) != 0 || card.status & STATUS_NO_LEVEL != 0 {
            0
        } else {
            card.data.level
        }
    }

    pub fn get_type(self, duel: &Duel) -> u32 {
        duel.get_type(self)
    }

    pub fn get_original_type(self, duel: &Duel) -> u32 {
        duel.cards[self].data.card_type
    }

    pub fn is_original_type(self, duel: &Duel, card_type: u32) -> bool {
        duel.cards[self].data.card_type & card_type != 0
    }

    // Card.IsExactType of the core, which utility.lua renames Card.IsCompositeType.
    pub fn is_composite_type(self, duel: &Duel, card_type: u32) -> bool {
        duel.get_type(self) & card_type == card_type
    }

    pub fn get_original_code(self, duel: &Duel) -> u32 {
        let data = &duel.cards[self].data;
        if data.alias != 0 && data.code.abs_diff(data.alias) < 10 {
            data.alias
        } else {
            data.code
        }
    }

    pub fn is_original_code(self, duel: &Duel, code: u32) -> bool {
        self.get_original_code(duel) == code
    }

    // The second code of an EFFECT_ADD_CODE comes with the cards that register one.
    pub fn get_original_code_rule(self, duel: &Duel) -> u32 {
        let added = duel.filter_effect(self, EFFECT_ADD_CODE, true);
        unported!(duel, added.iter());
        let data = &duel.cards[self].data;
        if data.alias != 0 && added.is_empty() {
            data.alias
        } else {
            data.code
        }
    }

    pub fn is_original_code_rule(self, duel: &Duel, code: u32) -> bool {
        self.get_original_code_rule(duel) == code
    }

    pub fn get_attribute(self, duel: &Duel) -> u32 {
        duel.get_attribute(self)
    }

    pub fn is_attribute(self, duel: &Duel, attribute: u32) -> bool {
        self.is_attribute_full(duel, attribute, None, 0, PLAYER_NONE)
    }

    // Attribute effects that apply only as material come with the cards that register them, so `scard` changes nothing.
    pub fn is_attribute_full(self, duel: &Duel, attribute: u32, _scard: Option<CardId>, _sumtype: u64, _player: u8) -> bool {
        duel.get_attribute(self) & attribute != 0
    }

    pub fn get_original_attribute(self, duel: &Duel) -> u32 {
        let card = &duel.cards[self];
        if card.status & STATUS_NO_LEVEL != 0 {
            0
        } else {
            card.data.attribute
        }
    }

    pub fn is_original_attribute(self, duel: &Duel, attribute: u32) -> bool {
        let card = &duel.cards[self];
        card.status & STATUS_NO_LEVEL == 0 && card.data.attribute & attribute != 0
    }

    pub fn get_race(self, duel: &Duel) -> u64 {
        duel.get_race(self)
    }

    pub fn is_race(self, duel: &Duel, race: u64) -> bool {
        self.is_race_full(duel, race, None, 0, PLAYER_NONE)
    }

    pub fn is_race_full(self, duel: &Duel, race: u64, _scard: Option<CardId>, _sumtype: u64, _player: u8) -> bool {
        duel.get_race(self) & race != 0
    }

    // Lua passes a table of archetypes or a single one; `setcodes` holds them.
    pub fn is_set_card(self, duel: &Duel, setcodes: &[u16]) -> bool {
        setcodes.iter().any(|&setcode| duel.is_set_card(self, setcode))
    }

    // Card.IsSetCard as material: card::get_summon_set_card, whose code and archetype effects with an operation come with
    // the cards that register them.
    pub fn is_set_card_full(self, duel: &Duel, setcodes: &[u16], _scard: Option<CardId>, _sumtype: u64, _player: u8) -> bool {
        let codes = [EFFECT_REMOVE_CODE, EFFECT_CHANGE_SETCODE];
        unported!(duel, codes.into_iter().flat_map(|code| duel.filter_effect(self, code, true)));
        self.is_set_card(duel, setcodes)
    }

    pub fn get_left_scale(self, duel: &Duel) -> u32 {
        duel.get_scale(self, false)
    }

    pub fn get_right_scale(self, duel: &Duel) -> u32 {
        duel.get_scale(self, true)
    }

    pub fn get_summon_type(self, duel: &Duel) -> u32 {
        duel.cards[self].summon.summon_type & 0xff00_ffff
    }

    pub fn is_summon_location(self, duel: &Duel, location: u32) -> bool {
        let summon = duel.cards[self].summon;
        let state = crate::card::CardState {
            location: summon.location,
            sequence: summon.sequence,
            pzone: summon.pzone,
            ..crate::card::CardState::default()
        };
        state.is_location(location as u16)
    }

    pub fn get_summon_player(self, duel: &Duel) -> u8 {
        duel.cards[self].summon.player
    }

    pub fn is_summon_player(self, duel: &Duel, player: u8) -> bool {
        duel.cards[self].summon.player == player
    }

    pub fn is_public(self, duel: &Duel) -> bool {
        duel.cards[self].is_position(POS_FACEUP)
    }

    pub fn is_forbidden(self, duel: &Duel) -> bool {
        duel.cards[self].is_status(STATUS_FORBIDDEN)
    }

    pub fn check_unique_on_field(self, duel: &Duel, player: u8) -> bool {
        duel.check_unique_onfield(self, player, LOCATION_ONFIELD).is_none()
    }

    pub fn is_has_effect(self, duel: &Duel, code: u32) -> Option<EffectId> {
        duel.filter_effect(self, code, true).first().copied()
    }

    pub fn get_equip_target(self, duel: &Duel) -> Option<CardId> {
        duel.cards[self].equiping_target
    }

    pub fn get_equip_group(self, duel: &Duel) -> CardSet {
        duel.equiping_cards(self)
    }

    // The counts of equipped Union monsters in their union state and in the old union state.
    pub fn get_union_count(self, duel: &Duel) -> (usize, usize) {
        let unions = |code| {
            duel.equiping_cards(self)
                .iter()
                .filter(|&equip| duel.cards[equip].data.card_type & TYPE_UNION != 0 && duel.is_affected_by_effect(equip, code).is_some())
                .count()
        };
        (unions(EFFECT_UNION_STATUS), unions(EFFECT_OLDUNION_STATUS))
    }

    pub fn set_card_target(self, duel: &mut Duel, target: CardId) {
        duel.add_card_target(self, target);
    }

    pub fn get_card_target(self, duel: &Duel) -> CardSet {
        duel.card_targets(self)
    }

    pub fn get_first_card_target(self, duel: &Duel) -> Option<CardId> {
        duel.card_targets(self).get_first()
    }

    pub fn is_has_card_target(self, duel: &Duel, target: CardId) -> bool {
        duel.card_targets.contains(&(self, target))
    }

    pub fn create_relation(self, duel: &mut Duel, target: CardId, reset: u32) {
        if !duel
            .card_relations
            .iter()
            .any(|&(card, related, _)| card == self && related == target)
        {
            duel.card_relations.push((self, target, reset));
        }
    }

    pub fn release_relation(self, duel: &mut Duel, target: CardId) {
        duel.card_relations
            .retain(|&(card, related, _)| !(card == self && related == target));
    }

    pub fn is_relate_to_card(self, duel: &Duel, target: CardId) -> bool {
        duel.card_relations
            .iter()
            .any(|&(card, related, _)| card == self && related == target)
    }

    // Card.IsGeminiState and Card.EnableGeminiState, under the names proc_gemini.lua gives them.
    pub fn is_gemini_status(self, duel: &Duel) -> bool {
        duel.is_affected_by_effect(self, EFFECT_GEMINI_STATUS).is_some()
    }

    pub fn enable_gemini_status(self, duel: &mut Duel) {
        duel.add_status_effect(self, EFFECT_GEMINI_STATUS, 64, RESET_EVENT + 0x1fe_0000);
    }

    pub fn get_card_effect(self, duel: &Duel, code: u32) -> EffectSet {
        duel.get_card_effect(self, code)
    }

    // The old-negate rule (DUEL_SPSUMMON_ONCE_OLD_NEGATE) and its global flag are not part of the duel mode.
    pub fn set_sp_summon_once(self, duel: &mut Duel, code: u32) {
        if duel.cards[self].is_status(STATUS_COPYING_EFFECT) {
            return;
        }
        duel.cards[self].spsummon_code = code;
    }

    pub fn is_relate_to_battle(self, duel: &Duel) -> bool {
        let fieldid = duel.cards[self].fieldid_r;
        duel.core.pre_field.contains(&fieldid)
    }

    pub fn can_chain_attack(self, duel: &mut Duel) -> bool {
        self.can_chain_attack_full(duel, 2, false)
    }

    pub fn can_chain_attack_full(self, duel: &mut Duel, ac: u8, monsteronly: bool) -> bool {
        if Some(self) != duel.core.attacker {
            return false;
        }
        let card = &duel.cards[self];
        if card.is_status(STATUS_BATTLE_DESTROYED)
            || card.current.controler != duel.infos.turn_player
            || card.fieldid_r != duel.core.pre_field[0]
            || !duel.is_capable_attack_announce(self, duel.infos.turn_player)
            || (ac != 0 && card.announce_count >= ac)
            || (ac == 2 && duel.is_affected_by_effect(self, EFFECT_EXTRA_ATTACK).is_some())
        {
            return false;
        }
        let targets = duel.field_get_attack_target(self, true, true);
        !(targets.is_empty() && (monsteronly || duel.cards[self].direct_attackable == 0))
    }

    // A relation to the chain link of the effect if one is on the chain, else to the effect alone.
    pub fn create_effect_relation(self, duel: &mut Duel, effect: EffectId) {
        let chain_id = duel
            .core
            .current_chain
            .iter()
            .rev()
            .find(|chain| chain.triggering_effect == effect)
            .map_or(0, |chain| chain.chain_id);
        duel.create_relation(self, effect, chain_id);
    }

    pub fn complete_procedure(self, duel: &mut Duel) {
        duel.cards[self].set_status(STATUS_PROC_COMPLETE, true);
    }

    pub fn set_material(self, duel: &mut Duel, materials: Option<CardSet>) {
        duel.set_material(self, materials.unwrap_or_default());
    }

    pub fn enable_revive_limit(self, duel: &mut Duel) {
        if duel.cards[self].is_status(STATUS_COPYING_EFFECT) {
            return;
        }
        for code in [EFFECT_UNSUMMONABLE_CARD, EFFECT_REVIVE_LIMIT] {
            let effect = duel.new_effect(self);
            let peffect = &mut duel.effects[effect];
            peffect.code = code;
            peffect.effect_type = EFFECT_TYPE_SINGLE;
            peffect.flag[0] = EFFECT_FLAG_CANNOT_DISABLE | EFFECT_FLAG_UNCOPYABLE;
            duel.card_add_effect(self, effect);
        }
    }

    pub fn is_able_to_deck_core(self, duel: &Duel) -> bool {
        duel.is_capable_send_to_deck(self, duel.core.reason_player.get())
    }

    pub fn is_able_to_grave_core(self, duel: &Duel) -> bool {
        duel.is_capable_send_to_grave(self, duel.core.reason_player.get(), REASON_EFFECT)
    }

    pub fn is_able_to_remove_core(self, duel: &Duel, player: u8, position: u32, reason: u32) -> bool {
        duel.is_removeable(self, player, position as u8, reason)
    }

    pub fn is_able_to_grave_as_cost(self, duel: &Duel) -> bool {
        duel.is_capable_cost_to_grave(self, duel.core.reason_player.get())
    }

    pub fn is_able_to_hand_as_cost(self, duel: &Duel) -> bool {
        duel.is_capable_cost_to_hand(self, duel.core.reason_player.get())
    }

    pub fn is_able_to_deck_as_cost(self, duel: &Duel) -> bool {
        duel.is_capable_cost_to_deck(self, duel.core.reason_player.get())
    }

    pub fn is_able_to_extra_as_cost(self, duel: &Duel) -> bool {
        duel.is_capable_cost_to_extra(self, duel.core.reason_player.get())
    }

    pub fn is_able_to_remove_as_cost(self, duel: &Duel) -> bool {
        self.is_able_to_remove_as_cost_full(duel, POS_FACEUP)
    }

    pub fn is_able_to_remove_as_cost_full(self, duel: &Duel, position: u32) -> bool {
        duel.is_removeable_as_cost(self, duel.core.reason_player.get(), position as u8)
    }

    pub fn is_synchro_summonable(self, duel: &Duel, must: Option<CardSet>, materials: Option<CardSet>) -> bool {
        self.is_synchro_summonable_full(duel, must, materials, 0, 0)
    }

    pub fn is_synchro_summonable_full(self, duel: &Duel, must: Option<CardSet>, materials: Option<CardSet>, min: u16, max: u16) -> bool {
        duel.spsummonable_rule(self, TYPE_SYNCHRO, SUMMON_TYPE_SYNCHRO, must, materials, min, max)
    }

    pub fn is_link_summonable(self, duel: &Duel, must: Option<CardSet>, materials: Option<CardSet>) -> bool {
        self.is_link_summonable_full(duel, must, materials, 0, 0)
    }

    pub fn is_link_summonable_full(self, duel: &Duel, must: Option<CardSet>, materials: Option<CardSet>, min: u16, max: u16) -> bool {
        duel.spsummonable_rule(self, TYPE_LINK, SUMMON_TYPE_LINK, must, materials, min, max)
    }

    pub fn set_reason_player(self, duel: &mut Duel, player: u8) {
        if player <= PLAYER_ALL {
            duel.cards[self].current.reason_player = player;
        }
    }

    // The label of the first flag effect of the code, as the first of the values Lua gets.
    pub fn get_flag_effect_label(self, duel: &Duel, code: u32) -> Option<i64> {
        let code = (code & 0xfff_ffff) | 0x1000_0000;
        let effect = duel
            .field_effects
            .registry
            .equal_range(Container::Single, Some(self), code)
            .next()?;
        Some(duel.effects[effect].labels().first().copied().unwrap_or(0))
    }

    pub fn get_flag_effect(self, duel: &Duel, code: u32) -> usize {
        let code = (code & 0xfff_ffff) | 0x1000_0000;
        duel.field_effects.registry.equal_range(Container::Single, Some(self), code).count()
    }

    pub fn reset_flag_effect(self, duel: &mut Duel, code: u32) {
        duel.reset(self, (code & 0xfff_ffff) | 0x1000_0000, RESET_CODE);
    }
}

impl CardId {
    // Card.AssumeProperty, called inside checks. Only card::get_race reads an assumed property so far; the other kinds
    // (and the invalid ones the core rejects with a Lua error) come with the cards that assume them.
    pub fn assume_property(self, duel: &Duel, assume: u32, value: u64) {
        if assume != ASSUME_RACE {
            duel.reach_unported(concat!(file!(), ":", line!()));
            return;
        }
        let mut assumes = duel.assumes.borrow_mut();
        match assumes
            .iter_mut()
            .find(|(card, kind, _)| (*card, u32::from(*kind)) == (self, assume))
        {
            Some(entry) => entry.2 = value,
            None => assumes.push((self, assume as u8, value)),
        }
    }

    // card::get_linked_cards.
    pub fn get_linked_group(self, duel: &Duel) -> CardSet {
        let card = &duel.cards[self];
        if card.data.card_type & TYPE_LINK == 0
            || card.current.location & LOCATION_ONFIELD == 0
            || card.get_status(STATUS_SUMMONING | STATUS_SPSUMMON_STEP)
        {
            return CardSet::default();
        }
        let (zone, p) = (duel.card_linked_zone(&duel.zones(), self), card.current.controler);
        let mut cards = CardSet::default();
        for (p, zone) in [(p, zone), (1 - p, zone >> 16)] {
            let player = &duel.player[p];
            let mzone = player.list_mzone.iter().enumerate().filter(|&(s, _)| zone & 1 << s != 0);
            let szone = player.list_szone.iter().enumerate().filter(|&(s, _)| zone & 1 << (8 + s) != 0);
            let szone = szone
                .filter_map(|(_, card)| *card)
                .filter(|&card| duel.get_type(card) & TYPE_LINK != 0);
            mzone.filter_map(|(_, card)| *card).chain(szone).for_each(|card| cards.insert(card));
        }
        cards
    }

    // Card.GetFreeLinkedZone for the controller: card::get_linked_zone(true).
    pub fn get_free_linked_zone(self, duel: &Duel) -> u32 {
        let controler = duel.cards[self].current.controler;
        let mut zone = duel.card_linked_zone(&duel.zones(), self);
        let sides = [
            (controler, 0, LOCATION_MZONE),
            (controler, 8, LOCATION_SZONE),
            (1 - controler, 16, LOCATION_MZONE),
            (1 - controler, 24, LOCATION_SZONE),
        ];
        for (player, shift, location) in sides {
            for sequence in 0..8 {
                if !duel.is_location_useable(player, u16::from(location), sequence) {
                    zone &= !(1 << (sequence + shift));
                }
            }
        }
        zone
    }

    // Card.GetLinkedZone with the player the zones are seen from.
    pub fn get_linked_zone_full(self, duel: &Duel, player: u8) -> u32 {
        let zone = duel.card_linked_zone(&duel.zones(), self);
        if player == 1u8.wrapping_sub(duel.cards[self].current.controler) {
            ((zone & 0xffff) << 16) | (zone >> 16)
        } else {
            zone
        }
    }

    pub fn get_reason_card(self, duel: &Duel) -> Option<CardId> {
        duel.cards[self].current.reason_card
    }

    pub fn set_hint(self, duel: &mut Duel, hint_type: u8, value: u64) {
        if hint_type >= CHINT_DESC_ADD {
            return;
        }
        let location = duel.cards.get_info_location(self);
        duel.trace.message(MSG_CARD_HINT).loc(location).u8(hint_type).u64(value);
    }

    pub fn is_not_tuner(self, duel: &Duel, scard: CardId, player: u8) -> bool {
        duel.is_not_tuner(self, Some(scard), player)
    }

    pub fn get_synchro_level(self, duel: &Duel, _scard: CardId) -> u32 {
        duel.get_synchro_level(self)
    }

    pub fn is_can_be_synchro_material(self, duel: &Duel, scard: CardId) -> bool {
        self.is_can_be_synchro_material_full(duel, Some(scard), None, None)
    }

    // `player` None is the reason player.
    pub fn is_can_be_synchro_material_full(self, duel: &Duel, scard: Option<CardId>, _tuner: Option<CardId>, player: Option<u8>) -> bool {
        let player = player.unwrap_or_else(|| duel.core.reason_player.get());
        duel.is_can_be_synchro_material(self, scard, player)
    }

    pub fn is_can_be_link_material(self, duel: &Duel, scard: Option<CardId>, player: Option<u8>) -> bool {
        let player = player.unwrap_or_else(|| duel.core.reason_player.get());
        duel.is_can_be_link_material(self, scard, player)
    }

    pub fn get_link(self, duel: &Duel) -> u32 {
        duel.get_link(self)
    }

    pub fn get_ritual_level(self, duel: &Duel, rc: CardId) -> u32 {
        duel.get_ritual_level(self, rc)
    }

    // The player is the reason player.
    pub fn is_can_be_ritual_material(self, duel: &Duel, sc: Option<CardId>) -> bool {
        duel.is_can_be_ritual_material(self, sc, duel.core.reason_player.get())
    }

    pub fn get_material(self, duel: &Duel) -> CardSet {
        duel.material_cards(self)
    }

    pub fn get_material_count(self, duel: &Duel) -> usize {
        duel.material_cards(self).len()
    }

    // `sumtype` is SUMMON_TYPE_FUSION where Lua leaves it out; the player is the reason player.
    pub fn is_can_be_fusion_material(self, duel: &Duel, fc: Option<CardId>, sumtype: u64) -> bool {
        duel.is_can_be_fusion_material(self, fc, sumtype, duel.core.reason_player.get())
    }

    pub fn check_fusion_material(self, duel: &Duel, g: Option<CardSet>, gc: Option<CardSet>, chkf: u64) -> bool {
        duel.fusion_materials_valid(self, g, gc, chkf).next().is_some()
    }

    pub fn check_fusion_substitute(self, duel: &Duel, fc: CardId) -> bool {
        duel.check_fusion_substitute(self, fc)
    }

    // Card.IsSummonCode(c, sc, sumtype, playerid, ...): sc, sumtype and playerid only reach code effects with an
    // operation, which are not ported.
    pub fn is_summon_code(self, duel: &Duel, _sc: Option<CardId>, _sumtype: u64, _player: u8, codes: &[u32]) -> bool {
        duel.is_summon_code(self, codes)
    }

    pub fn is_able_to_extra_core(self, duel: &Duel) -> bool {
        duel.is_capable_send_to_extra(self, duel.core.reason_player.get())
    }

    pub fn get_base_attack(self, duel: &Duel) -> i32 {
        duel.get_base_attack(self).max(0)
    }

    pub fn get_base_defense(self, duel: &Duel) -> i32 {
        duel.get_base_defense(self).max(0)
    }

    pub fn get_defense(self, duel: &Duel) -> i32 {
        duel.get_defense(self).max(0)
    }

    pub fn get_battle_target(self, duel: &Duel) -> Option<CardId> {
        if duel.core.attacker == Some(self) {
            duel.core.attack_target
        } else if duel.core.attack_target == Some(self) {
            duel.core.attacker
        } else {
            None
        }
    }

    pub fn get_turn_id(self, duel: &Duel) -> i32 {
        i32::from(duel.cards[self].turnid)
    }

    pub fn get_reason_player(self, duel: &Duel) -> u8 {
        duel.cards[self].current.reason_player
    }

    pub fn is_reason(self, duel: &Duel, reason: u32) -> bool {
        duel.cards[self].current.reason & reason != 0
    }

    // Lua passes the summon types to match as further arguments.
    pub fn is_summon_type(self, duel: &Duel, summon_types: &[u32]) -> bool {
        let summon_type = duel.cards[self].summon.summon_type & 0xff00_ffff;
        summon_types.iter().any(|&wanted| wanted & !summon_type == 0)
    }

    pub fn is_ssetable(self, duel: &Duel) -> bool {
        self.is_ssetable_full(duel, false, None)
    }

    // `player` None is the reason player.
    pub fn is_ssetable_full(self, duel: &Duel, ignore_field: bool, player: Option<u8>) -> bool {
        let player = player.unwrap_or_else(|| duel.core.reason_player.get());
        duel.is_setable_szone(self, player, ignore_field)
    }

    pub fn is_level_above(self, duel: &Duel, level: u32) -> bool {
        let current = duel.get_level(self);
        current > 0 && current >= level
    }

    fn is_monster_now(self, duel: &Duel) -> bool {
        duel.cards[self].data.card_type & TYPE_MONSTER != 0
            || duel.get_type(self) & TYPE_MONSTER != 0
            || duel.is_affected_by_effect(self, EFFECT_PRE_MONSTER).is_some()
    }

    pub fn is_attack_above(self, duel: &Duel, attack: i32) -> bool {
        self.is_monster_now(duel) && duel.get_attack(self) >= attack
    }

    pub fn is_defense_above(self, duel: &Duel, defense: i32) -> bool {
        duel.cards[self].data.card_type & TYPE_LINK == 0 && self.is_monster_now(duel) && duel.get_defense(self) >= defense
    }

    // Lua passes the values to match as further arguments.
    pub fn is_attack(self, duel: &Duel, attacks: &[i32]) -> bool {
        self.is_monster_now(duel) && attacks.contains(&duel.get_attack(self))
    }

    pub fn get_text_attack(self, duel: &Duel) -> i32 {
        let card = &duel.cards[self];
        if card.status & STATUS_NO_LEVEL != 0 { 0 } else { card.data.attack }
    }

    pub fn get_original_race(self, duel: &Duel) -> u64 {
        let card = &duel.cards[self];
        if card.status & STATUS_NO_LEVEL != 0 { 0 } else { card.data.race }
    }

    pub fn is_original_race(self, duel: &Duel, race: u64) -> bool {
        self.get_original_race(duel) & race != 0
    }

    // Lua passes the values to match as further arguments.
    pub fn is_defense(self, duel: &Duel, defenses: &[i32]) -> bool {
        duel.cards[self].data.card_type & TYPE_LINK == 0 && self.is_monster_now(duel) && defenses.contains(&duel.get_defense(self))
    }

    pub fn is_releasable(self, duel: &Duel) -> bool {
        self.is_releasable_full(duel, REASON_COST, None)
    }

    // `player` None is the reason player.
    pub fn is_releasable_full(self, duel: &Duel, reason: u32, player: Option<u8>) -> bool {
        let player = player.unwrap_or_else(|| duel.core.reason_player.get());
        duel.is_releasable_by_nonsummon(self, player, reason)
    }

    pub fn is_releasable_by_effect(self, duel: &Duel) -> bool {
        self.is_releasable_by_effect_full(duel, None, None)
    }

    // `effect` and `player` None are the reason effect and player.
    pub fn is_releasable_by_effect_full(self, duel: &Duel, effect: Option<EffectId>, player: Option<u8>) -> bool {
        let effect = effect.or(duel.core.reason_effect.get());
        let player = player.unwrap_or_else(|| duel.core.reason_player.get());
        duel.is_releasable_by_nonsummon(self, player, REASON_EFFECT) && duel.is_releasable_by_effect(self, effect)
    }

    pub fn is_immune_to_effect(self, duel: &Duel, effect: EffectId) -> bool {
        !duel.is_affect_by_effect(self, Some(effect))
    }

    pub fn is_can_be_disabled_by_effect(self, duel: &Duel, effect: EffectId, is_monster_effect: bool) -> bool {
        let card = &duel.cards[self];
        if card.is_status(STATUS_DISABLED) && (is_monster_effect || duel.get_type(self) & TYPE_TRAPMONSTER == 0) {
            return false;
        }
        duel.is_affected_by_effect(self, EFFECT_CANNOT_DISABLE).is_none() && duel.is_affect_by_effect(self, Some(effect))
    }

    pub fn is_destructable(self, duel: &Duel) -> bool {
        duel.is_destructable(self)
    }

    pub fn is_summonable_card(self, duel: &Duel) -> bool {
        duel.is_summonable_card(self)
    }

    pub fn get_column_group(self, duel: &Duel) -> CardSet {
        self.get_column_group_full(duel, 0, 0)
    }

    pub fn get_column_group_full(self, duel: &Duel, left: u8, right: u8) -> CardSet {
        duel.get_column_cards(self, i32::from(left), i32::from(right))
    }

    pub fn get_counter(self, duel: &Duel, counter_type: u16) -> u16 {
        duel.get_counter(self, counter_type)
    }

    pub fn add_counter(self, duel: &mut Duel, counter_type: u16, count: u16) -> bool {
        self.add_counter_full(duel, counter_type, count, false)
    }

    pub fn add_counter_full(self, duel: &mut Duel, counter_type: u16, count: u16, singly: bool) -> bool {
        let (reason_effect, reason_player) = (duel.core.reason_effect.get(), duel.core.reason_player.get());
        duel.is_affect_by_effect(self, reason_effect) && duel.add_counter(self, reason_player, counter_type, count, singly)
    }

    pub fn is_can_add_counter(self, duel: &Duel, counter_type: u16, count: u16) -> bool {
        duel.is_can_add_counter(self, duel.core.reason_player.get(), counter_type, count, false, 0)
    }

    pub fn enable_counter_permit(self, duel: &mut Duel, counter_type: u16) {
        self.enable_counter_permit_full(duel, counter_type, None);
    }

    // `range` None is the monster zone for a monster, the spell zones otherwise; a target filter comes with the cards
    // that pass one.
    pub fn enable_counter_permit_full(self, duel: &mut Duel, counter_type: u16, range: Option<u16>) {
        let range = range.unwrap_or(if duel.cards[self].data.card_type & TYPE_MONSTER != 0 {
            u16::from(LOCATION_MZONE)
        } else {
            u16::from(LOCATION_SZONE) | LOCATION_FZONE
        });
        let effect = duel.new_effect(self);
        let peffect = &mut duel.effects[effect];
        peffect.effect_type = EFFECT_TYPE_SINGLE;
        peffect.code = EFFECT_COUNTER_PERMIT | u32::from(counter_type);
        peffect.value = crate::effect::Value::Int(i32::from(range));
        duel.card_add_effect(self, effect);
    }
}

impl CardId {
    pub fn is_previous_controler(self, duel: &Duel, player: u8) -> bool {
        duel.cards[self].previous.controler == player
    }
}

impl CardId {
    pub fn get_location(self, duel: &Duel) -> u32 {
        let card = &duel.cards[self];
        if card.get_status(STATUS_SUMMONING | STATUS_SUMMON_DISABLED | STATUS_ACTIVATE_DISABLED | STATUS_SPSUMMON_STEP) {
            0
        } else {
            u32::from(card.current.location)
        }
    }
}

impl CardId {
    pub fn get_rank(self, duel: &Duel) -> u32 {
        duel.get_rank(self)
    }

    // Lua passes the ranks to match as further arguments.
    pub fn is_rank(self, duel: &Duel, ranks: &[u32]) -> bool {
        ranks.contains(&duel.get_rank(self))
    }

    pub fn is_xyz_level(self, duel: &Duel, xyzc: CardId, level: u32) -> bool {
        duel.check_xyz_level(self, xyzc, level)
    }

    // `player` and `reason` are those Lua defaults to when a call leaves them out: the reason player and
    // REASON_XYZ | REASON_MATERIAL.
    pub fn is_can_be_xyz_material(self, duel: &Duel, scard: Option<CardId>, player: u8, reason: u32) -> bool {
        duel.is_can_be_xyz_material(self, scard, player, reason)
    }

    pub fn get_overlay_group(self, duel: &Duel) -> CardSet {
        duel.xyz_materials(self)
    }

    pub fn get_overlay_count(self, duel: &Duel) -> usize {
        duel.xyz_materials(self).len()
    }

    pub fn check_remove_overlay_card(self, duel: &Duel, player: u8, count: u16, _reason: u32) -> bool {
        player <= 1 && duel.is_player_can_remove_overlay_card(player, Some(self.into()), 0, 0, count)
    }

    pub fn remove_overlay_card(self, duel: &mut Duel, player: u8, min: u16, max: u16, reason: u32) -> Script<i32> {
        if player > 1 {
            return Ok(0);
        }
        duel.field_remove_overlay_card(reason, Some(self.into()), player, 0, 0, min, max);
        duel.yield_script()?;
        Ok(duel.returns.at_i32(0))
    }
}
