// Centur-Ion Bonds
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND};

const ID: u32 = 4160316;

pub const LISTED_SERIES: &[u16] = &[SET_CENTUR_ION];
// Centur-Ion Trudea and Primera to place, and Centur-Ion Legatia in the Graveyard for Monster Reborn and Centur-Ion Phalanx to Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (ID, GRAVE),
    (42493140, GRAVE),
    (15005145, HAND),
    (15982593, GRAVE),
    (71858682, GRAVE),
    (83764718, HAND),
    (40155014, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property_full(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| cfilter(c, duel, a.tp), 1, None))
    });
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, pltg);
    e2.set_operation(duel, plop);
    c.register_effect(duel, e2);
}

fn move_to_stzone(duel: &mut Duel, c: CardId, hc: CardId, tp: u8) -> Script<bool> {
    if !duel.move_to_field(c, tp, tp, LOCATION_SZONE, POS_FACEUP, true)? {
        return Ok(false);
    }
    let e1 = duel.create_effect(hc);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EFFECT_CHANGE_TYPE);
    e1.set_value(duel, (TYPE_TRAP | TYPE_CONTINUOUS) as i32);
    e1.set_reset(duel, (RESET_EVENT | RESETS_STANDARD) & !RESET_TURN_SET);
    c.register_effect(duel, e1);
    Ok(true)
}

fn plfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_CENTUR_ION]) && c.is_monster(duel) && !c.is_forbidden(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let mut ft = duel.get_location_count(tp, LOCATION_SZONE);
    if a.e.get_handler(duel).is_location(duel, LOCATION_HAND) {
        ft -= 1;
    }
    if chk == 0 {
        return Ok(ft > 0 && duel.is_existing_matching_card(plfilter, tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_possible_operation_info(0, CATEGORY_LEAVE_GRAVE, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.get_location_count(tp, LOCATION_SZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(plfilter),
        tp,
        LOCATION_HAND | LOCATION_GRAVE,
        0,
        1,
        1,
        None,
    )?;
    if let Some(tc) = g.get_first() {
        let handler = a.e.get_handler(duel);
        move_to_stzone(duel, tc, handler, tp)?;
    }
    Ok(())
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel) && c.is_controler(duel, tp) && c.is_set_card(duel, &[SET_CENTUR_ION]) && c.is_type(duel, TYPE_SYNCHRO)
}

fn pltg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && plfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0 && duel.is_existing_target(plfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    let g = duel.select_target(tp, plfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, g, 1, 0, 0);
    Ok(false)
}

fn plop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && duel.get_location_count(tp, LOCATION_SZONE) > 0
    {
        let handler = a.e.get_handler(duel);
        move_to_stzone(duel, tc, handler, tp)?;
    }
    Ok(())
}
