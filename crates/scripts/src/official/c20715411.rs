// D/D/D Zero Doom Queen Machinex
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 20715411;

pub const LISTED_SERIES: &[u16] = &[SET_DARK_CONTRACT, SET_DDD];
pub const LISTED_NAMES: &[u32] = &[ID];
// In the hand to activate, with Dark Contract with the Witch in the Deck to place, and on the field with the opponent's
// Raigeki to be destroyed.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (ID, MZONE),
    (9765723, DECK),
    (9765723, DECK),
    (9765723, DECK),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure_full(duel, c, false, None);
    let e0 = duel.create_effect(c);
    e0.set_description(duel, 1160);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    e0.set_range(duel, LOCATION_HAND);
    e0.set_cost(duel, actreg);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).get_flag_effect(duel, ID) > 0);
    e1.set_target(duel, pltg);
    e1.set_operation(duel, plop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_range(duel, LOCATION_EXTRA);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).is_previous_location(duel, LOCATION_MZONE));
    e3.set_target(duel, |duel, a, chk, _| Ok(chk == 0 && duel.check_pendulum_zones(a.tp)));
    e3.set_operation(duel, penop);
    c.register_effect(duel, e3);
}

fn actreg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    a.e.get_handler(duel)
        .register_flag_effect(duel, ID, RESET_PHASE | PHASE_END, EFFECT_FLAG_OATH, 1);
    Ok(false)
}

fn plfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_DARK_CONTRACT])
        && c.is_continuous_spell_trap(duel)
        && !c.is_forbidden(duel)
        && c.check_unique_on_field(duel, tp)
}

fn pltg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    Ok(chk == 0
        && duel.get_location_count(tp, LOCATION_SZONE) > 0
        && duel.is_existing_matching_card(|c, duel| plfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, None))
}

fn plop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.get_location_count(tp, LOCATION_SZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    let g = duel.select_matching_card(tp, |c, duel| plfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first() {
        duel.move_to_field(sc, tp, tp, LOCATION_SZONE, POS_FACEUP, true)?;
    }
    Ok(())
}

fn spconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_previous_controler(duel, tp)
        && c.is_previous_location(duel, LOCATION_ONFIELD)
        && !c.is_previous_code_on_field(duel, &[ID])
        && c.is_previous_position(duel, POS_FACEUP)
        && c.is_previous_set_card(duel, &[SET_DDD, SET_DARK_CONTRACT])
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(eg) = a.eg else { return false };
    !eg.is_contains(a.e.get_handler(duel)) && eg.is_exists(duel, |c, duel| spconfilter(c, duel, a.tp), 1, None)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count_from_ex(tp, tp, None, c) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, None, 1, PLAYER_EITHER, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e)
        && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0
        && duel.is_existing_matching_card(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 3))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let g = duel.select_matching_card(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
        if !g.is_empty() {
            duel.hint_selection(g);
            duel.break_effect()?;
            duel.destroy(g, REASON_EFFECT)?;
        }
    }
    Ok(())
}

pub fn penop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.move_to_field(c, a.tp, a.tp, LOCATION_PZONE, POS_FACEUP, true)?;
    }
    Ok(())
}
