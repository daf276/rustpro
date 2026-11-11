// Dragon Spirit of White
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 45467446;

pub const LISTED_NAMES: &[u32] = &[CARD_BLUEEYES_W_DRAGON];

// On the field and in the hand with Blue-Eyes White Dragon in the hand to Special Summon, and an opponent's Spells and Traps to
// banish and monster to fight.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, HAND),
    (CARD_BLUEEYES_W_DRAGON, HAND),
    (4206964, SZONE | OPPONENT),
    (4206964, SZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::SpecialSummonedCode(CARD_BLUEEYES_W_DRAGON)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_ADD_TYPE);
    e1.set_range(duel, LOCATION_HAND | LOCATION_GRAVE);
    e1.set_value(duel, TYPE_NORMAL as i32);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_REMOVE_TYPE);
    e2.set_value(duel, TYPE_EFFECT as i32);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_category(duel, CATEGORY_REMOVE);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_SUMMON_SUCCESS);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_target(duel, rmtg);
    e3.set_operation(duel, rmop);
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e5.set_type(duel, EFFECT_TYPE_QUICK_O);
    e5.set_hint_timing(duel, 0, TIMING_END_PHASE);
    e5.set_code(duel, EVENT_FREE_CHAIN);
    e5.set_range(duel, LOCATION_MZONE);
    e5.set_condition(duel, |duel, a| duel.get_field_group_count(a.tp, 0, LOCATION_MZONE) > 0);
    e5.set_cost(duel, Cost::self_tribute);
    e5.set_target(duel, sptg);
    e5.set_operation(duel, spop);
    c.register_effect(duel, e5);
}

fn rmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel) && c.is_able_to_remove(duel)
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_ONFIELD) && chkc.is_controler(duel, 1 - tp) && rmfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(rmfilter, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, rmfilter, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, 0, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.remove(tc, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, CARD_BLUEEYES_W_DRAGON) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    if e.get_handler(duel).get_sequence(duel) < 5 {
        ft += 1;
    }
    if chk == 0 {
        return Ok(ft > 0 && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_HAND, POS_FACEUP)
}
