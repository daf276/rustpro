// Port of proc_workaround.lua: the Card methods it replaces and the global effects it registers on load.
use engine::{Args, CardId, CardSet, Duel, EffectId, Script};

use crate::chain::DuelChain;
use crate::constant::*;
use crate::utility::CardUtility;

pub trait CardWorkaround: Copy {
    fn is_relate_to_effect(self, duel: &Duel, e: EffectId) -> bool;
    fn is_able_to_hand(self, duel: &Duel) -> bool;
    fn is_able_to_deck(self, duel: &Duel) -> bool;
    fn is_able_to_extra(self, duel: &Duel) -> bool;
    fn is_able_to_grave(self, duel: &Duel) -> bool;
    fn is_able_to_remove(self, duel: &Duel) -> bool;
    // Card.IsAbleToRemove(c, p, pos, reason); p None is the reason player.
    fn is_able_to_remove_full(self, duel: &Duel, player: Option<u8>, position: u32, reason: u32) -> bool;
}

impl CardWorkaround for CardId {
    fn is_relate_to_effect(self, duel: &Duel, e: EffectId) -> bool {
        if !(duel.is_chain_solving() && e.is_activated(duel)) {
            return self.is_relate_to_effect_core(duel, e);
        }
        if duel.get_reason_effect() == Some(e) {
            return self.is_relate_to_chain(duel, 0);
        }
        for chain_link in 0..duel.get_current_chain() {
            if duel.get_chain_info::<Option<EffectId>>(chain_link, CHAININFO_TRIGGERING_EFFECT) == Some(e) {
                return self.is_relate_to_chain(duel, chain_link);
            }
        }
        self.is_relate_to_effect_core(duel, e)
    }

    fn is_able_to_hand(self, duel: &Duel) -> bool {
        !self.is_location(duel, LOCATION_HAND) && self.is_able_to_hand_core(duel)
    }

    fn is_able_to_deck(self, duel: &Duel) -> bool {
        !self.is_location(duel, LOCATION_DECK) && self.is_able_to_deck_core(duel)
    }

    fn is_able_to_extra(self, duel: &Duel) -> bool {
        !self.is_location(duel, LOCATION_EXTRA) && self.is_able_to_extra_core(duel)
    }

    fn is_able_to_grave(self, duel: &Duel) -> bool {
        !self.is_location(duel, LOCATION_GRAVE) && self.is_able_to_grave_core(duel)
    }

    fn is_able_to_remove(self, duel: &Duel) -> bool {
        self.is_able_to_remove_full(duel, None, POS_FACEUP, REASON_EFFECT)
    }

    fn is_able_to_remove_full(self, duel: &Duel, player: Option<u8>, position: u32, reason: u32) -> bool {
        let player = player.unwrap_or_else(|| duel.get_reason_player());
        !self.is_location(duel, LOCATION_REMOVED) && self.is_able_to_remove_core(duel, player, position, reason)
    }
}

pub trait DuelWorkaround {
    fn swap_sequence(&mut self, card1: CardId, card2: CardId) -> bool;
    fn move_to_field(&mut self, card: CardId, move_player: u8, player: u8, destination: u32, positions: u32, enable: bool) -> Script<bool>;
    fn is_damage_step(&self) -> bool;
    fn get_target_group(
        &self,
        filter: impl Fn(CardId, &Duel) -> bool,
        player: u8,
        location1: u32,
        location2: u32,
        exception: Option<CardSet>,
    ) -> CardSet;
    // Duel.IsMainPhase(player) and Duel.IsStandbyPhase(player): the phase, and that it is the player's turn.
    fn is_main_phase_full(&self, player: Option<u8>) -> bool;
    fn is_standby_phase_full(&self, player: Option<u8>) -> bool;
}

impl DuelWorkaround for Duel {
    fn swap_sequence(&mut self, card1: CardId, card2: CardId) -> bool {
        let (sequence1, sequence2) = (card1.get_sequence(self), card2.get_sequence(self));
        self.swap_sequence_core(card1, card2);
        card1.is_on_field(self) && card2.is_on_field(self) && card1.is_sequence(self, sequence2) && card2.is_sequence(self, sequence1)
    }

    fn move_to_field(&mut self, card: CardId, move_player: u8, player: u8, destination: u32, positions: u32, enable: bool) -> Script<bool> {
        let reason_player = if self.get_reason_effect().is_some() {
            self.get_reason_player()
        } else {
            PLAYER_NONE
        };
        card.set_reason_player(self, reason_player);
        self.move_to_field_core(card, move_player, player, destination, positions, enable)
    }

    fn is_damage_step(&self) -> bool {
        let current_phase = self.get_current_phase();
        current_phase == PHASE_DAMAGE || current_phase == PHASE_DAMAGE_CAL
    }

    // Duel.GetTargetGroup: the matching cards the reason effect can target.
    fn get_target_group(
        &self,
        filter: impl Fn(CardId, &Duel) -> bool,
        player: u8,
        location1: u32,
        location2: u32,
        exception: Option<CardSet>,
    ) -> CardSet {
        let re = self.get_reason_effect();
        unported!(self, re.is_none());
        let g = self.get_matching_group(filter, player, location1, location2, exception);
        re.map_or(g, |re| g.filter(self, |c, duel| c.is_can_be_effect_target(duel, re), None))
    }

    fn is_main_phase_full(&self, player: Option<u8>) -> bool {
        self.is_main_phase() && player.is_none_or(|player| self.is_turn_player(player))
    }

    fn is_standby_phase_full(&self, player: Option<u8>) -> bool {
        self.get_current_phase() == PHASE_STANDBY && player.is_none_or(|player| self.is_turn_player(player))
    }
}

// player_table: whether a player's hand was already shuffled in the current chain, as script state per player.
const HAND_SHUFFLED: u64 = 1;

pub fn load(duel: &mut Duel) {
    let hand_adjust_eff = duel.global_effect();
    hand_adjust_eff.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    hand_adjust_eff.set_code(duel, EVENT_DISCARD);
    hand_adjust_eff.set_count_limit(duel, 1, 0);
    hand_adjust_eff.set_condition(duel, |_, a| a.r & REASON_ADJUST > 0);
    hand_adjust_eff.set_promptless_operation(duel, track_chain_solving);
    duel.register_effect(hand_adjust_eff, 0);

    let sum_status_eff = duel.global_effect();
    sum_status_eff.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    sum_status_eff.set_code(duel, EVENT_CHANGE_POS);
    sum_status_eff.set_promptless_operation(duel, |duel, a| {
        for c in a.eg.unwrap_or_default().filter(duel, summon_status_filter, None) {
            c.set_status(duel, STATUS_SUMMON_TURN, false);
            c.set_status(duel, STATUS_FLIP_SUMMON_TURN, false);
            c.set_status(duel, STATUS_SPSUMMON_TURN, false);
            c.set_status(duel, STATUS_FORM_CHANGED, true);
        }
        Ok(())
    });
    duel.register_effect(sum_status_eff, 0);

    let set_turn_status_split = duel.global_effect();
    set_turn_status_split.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    set_turn_status_split.set_code(duel, EVENT_MSET);
    set_turn_status_split.set_promptless_operation(duel, |duel, a| {
        for c in a.eg.unwrap_or_default() {
            c.set_status(duel, STATUS_SUMMON_TURN, false);
            c.set_status(duel, STATUS_FORM_CHANGED, true);
        }
        Ok(())
    });
    duel.register_effect(set_turn_status_split, 0);

    let shuffle_eff = duel.global_effect();
    shuffle_eff.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    shuffle_eff.set_code(duel, EVENT_CHAIN_SOLVING);
    shuffle_eff.set_promptless_operation(duel, shuffle_on_resolution);
    duel.register_effect(shuffle_eff, 0);

    let tracking_eff = duel.global_effect();
    tracking_eff.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    tracking_eff.set_code(duel, EVENT_CHAIN_END);
    tracking_eff.set_promptless_operation(duel, |duel, _| {
        (0..2).for_each(|player| duel.set_script_state(HAND_SHUFFLED + player, 0));
        Ok(())
    });
    duel.register_effect(tracking_eff, 0);

    let ns_eff = duel.global_effect();
    ns_eff.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    ns_eff.set_code(duel, EVENT_SUMMON_SUCCESS);
    ns_eff.set_promptless_operation(duel, |duel, a| {
        for sc in a.eg.unwrap_or_default() {
            let phase = duel.get_current_phase();
            sc.register_flag_effect_full(duel, 160214042, RESET_EVENT | RESETS_STANDARD, 0, 1, phase.into(), 0);
        }
        Ok(())
    });
    duel.register_effect(ns_eff, 0);
    let sp_eff = ns_eff.clone_effect(duel);
    sp_eff.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    duel.register_effect(sp_eff, 0);
}

fn track_chain_solving(duel: &mut Duel, _: &Args) -> Script<()> {
    let chain_solving_eff = duel.global_effect();
    chain_solving_eff.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    chain_solving_eff.set_code(duel, EVENT_CHAIN_SOLVING);
    chain_solving_eff.set_count_limit(duel, 1, 0);
    chain_solving_eff.set_promptless_operation(duel, |duel, a| {
        a.e.reset(duel);
        let cannot_act_eff = duel.global_effect();
        cannot_act_eff.set_type(duel, EFFECT_TYPE_FIELD);
        cannot_act_eff.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
        cannot_act_eff.set_code(duel, EFFECT_CANNOT_ACTIVATE);
        cannot_act_eff.set_target_range(duel, 1, 1);
        cannot_act_eff.set_value(duel, 1);
        cannot_act_eff.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(cannot_act_eff, 0);
        Ok(())
    });
    chain_solving_eff.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(chain_solving_eff, 0);
    Ok(())
}

fn summon_status_filter(c: CardId, duel: &Duel) -> bool {
    c.is_facedown(duel)
        && c.is_previous_position(duel, POS_FACEUP)
        && c.is_location(duel, LOCATION_MZONE)
        && c.is_status(duel, STATUS_SUMMON_TURN | STATUS_FLIP_SUMMON_TURN | STATUS_SPSUMMON_TURN)
}

fn check_opinfo(duel: &Duel, ev: u8, category: u64, rc: CardId) -> bool {
    let contains = |cards: Option<CardSet>| cards.is_some_and(|cards| cards.is_contains(rc));
    contains(duel.get_operation_info(ev, category).and_then(|info| info.op_cards))
        || contains(duel.get_possible_operation_info(ev, category).and_then(|info| info.op_cards))
}

fn shuffle_on_resolution(duel: &mut Duel, a: &Args) -> Script<()> {
    let ev = a.ev as u8;
    if duel.get_chain_info::<u32>(ev, CHAININFO_TRIGGERING_LOCATION) != LOCATION_HAND {
        return Ok(());
    }
    let Some(re) = a.re else { return Ok(()) };
    let rc = re.get_handler(duel);
    let player = rc.get_controler(duel);
    if duel.get_script_state(HAND_SHUFFLED + u64::from(player)) != 0 {
        return Ok(());
    }
    if !(rc.is_relate_to_effect(duel, re) && rc.is_location(duel, LOCATION_HAND)) {
        return Ok(());
    }
    let moves = [
        CATEGORY_SPECIAL_SUMMON,
        CATEGORY_SUMMON,
        CATEGORY_TOGRAVE,
        CATEGORY_DESTROY,
        CATEGORY_REMOVE,
        CATEGORY_TODECK,
        CATEGORY_TOEXTRA,
        CATEGORY_EQUIP,
        CATEGORY_RELEASE,
    ];
    if moves.into_iter().any(|category| check_opinfo(duel, ev, category, rc)) {
        return Ok(());
    }
    duel.shuffle_hand(player)?;
    // move_eff and the effect it registers keep `player` as their label, where the Lua closures capture it.
    let move_eff = duel.create_effect(rc);
    move_eff.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    move_eff.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    move_eff.set_code(duel, EVENT_MOVE);
    move_eff.set_label(duel, player.into());
    move_eff.set_promptless_operation(duel, |duel, a| {
        let player = a.e.get_label(duel) as u8;
        let eff = duel.create_effect(a.e.get_handler(duel));
        eff.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        eff.set_code(duel, EVENT_CHAIN_SOLVED);
        eff.set_label(duel, player.into());
        eff.set_promptless_operation(duel, |duel, a| {
            duel.shuffle_hand(a.e.get_label(duel) as u8)?;
            a.e.reset(duel);
            Ok(())
        });
        eff.set_reset(duel, RESET_CHAIN);
        duel.register_effect(eff, player);
        a.e.reset(duel);
        Ok(())
    });
    move_eff.set_reset(duel, RESET_CHAIN);
    rc.register_effect(duel, move_eff);
    duel.set_script_state(HAND_SHUFFLED + u64::from(player), 1);
    Ok(())
}
