// Sky Striker Mecha - Shark Cannon
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, OPPONENT};

const ID: u32 = 51227866;

// In the hand with no monsters on the field, a monster of the opponent's in the Graveyard to banish or Special Summon, and Spells in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (26202165, GRAVE | OPPONENT),
    (26202165, GRAVE | OPPONENT),
    (53129443, GRAVE),
    (53129443, GRAVE),
    (53129443, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::SpecialSummonedCode(26202165)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_REMOVE + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_hint_timing(duel, 0, TIMING_END_PHASE);
    e1.set_condition(duel, |duel, a| {
        !duel.is_existing_matching_card(|c, duel| c.get_sequence(duel) < 5, a.tp, LOCATION_MZONE, 0, 1, None)
    });
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8, spchk: bool) -> bool {
    c.is_monster(duel)
        && (c.is_able_to_remove(duel)
            || (spchk && duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)))
}

fn spell_count_reached(duel: &Duel, tp: u8) -> bool {
    duel.get_matching_group_count(CardId::is_spell, tp, LOCATION_GRAVE, 0, None) >= 3
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let spchk = spell_count_reached(duel, tp);
    if chkc.is_some() {
        // The Lua passes the undefined global `c` to the filter, which errors.
        return Ok(false);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| filter(c, duel, e, tp, spchk), tp, 0, LOCATION_GRAVE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel
        .select_target(tp, |c, duel| filter(c, duel, e, tp, spchk), tp, 0, LOCATION_GRAVE, 1, 1, None)?
        .unwrap_or_default();
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(g), 1, tp, LOCATION_GRAVE as i32);
    duel.set_possible_operation_info(0, CATEGORY_REMOVE, Some(g), 1, tp, LOCATION_GRAVE as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(g), 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    if spell_count_reached(duel, tp)
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && tc.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && (!tc.is_able_to_remove(duel) || duel.select_yes_no(tp, aux::stringid(ID, 0))?)
    {
        duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_description(duel, 3206);
        e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CANNOT_ATTACK);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect_full(duel, e1, true);
    } else {
        duel.remove(tc, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}
