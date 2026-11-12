// Dragonbite
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 84462118;

// Copies in the hand to Normal Summon and on the field, with Basic Insect and Petit Moth (Insect Normal Monsters) to Special Summon and to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (89091579, HAND),
    (58192742, HAND),
    (58192742, MZONE),
    (89091579, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, (ID, 1));
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_LVCHANGE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 2));
    e2.set_cost(duel, lvcost);
    e2.set_target(duel, lvtg);
    e2.set_operation(duel, lvop);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_below(duel, 4) && c.is_race(duel, RACE_INSECT) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_HAND)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, e, tp), LOCATION_HAND, POS_FACEUP)
}

fn faceup_with_level(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.has_level(duel)
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    (!c.is_on_field(duel) || c.is_faceup(duel))
        && c.is_race(duel, RACE_INSECT)
        && c.is_level_below(duel, 4)
        && c.is_able_to_remove_as_cost(duel)
        && duel.is_existing_target(faceup_with_level, tp, LOCATION_MZONE, 0, 1, Some(c.into()))
        && aux::sp_elim_filter(c, duel, true, true)
}

fn lvcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let locations = LOCATION_HAND | LOCATION_MZONE | LOCATION_GRAVE;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| cfilter(c, duel, tp), tp, locations, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, |c, duel| cfilter(c, duel, tp), tp, locations, 0, 1, 1, None)?;
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    if let Some(tc) = g.get_first() {
        let level = tc.get_level(duel);
        a.e.set_label(duel, i64::from(level));
    }
    Ok(false)
}

fn lvtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && faceup_with_level(chkc, duel));
    }
    if chk == 0 {
        return Ok(true);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g = duel.select_target(tp, faceup_with_level, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_LVCHANGE, g, 1, 0, 0);
    Ok(false)
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && tc.is_faceup(duel)
    {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_LEVEL);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_value(duel, e.get_label(duel) as i32);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
