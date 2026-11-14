// Tensei Ryu-Ge Anva
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 56322832;

pub const LISTED_SERIES: &[u16] = &[SET_RYU_GE];
pub const LISTED_NAMES: &[u32] = &[ID];
// Copies in the hand, Genro Ryu-Ge Hakva on the field to Tribute, Ryu-Ge Realms in the Deck to place, and the opponent's
// Dark Hole to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (56499179, MZONE),
    (56499179, MZONE),
    (55154344, DECK),
    (28669235, DECK),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(
        duel,
        EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CANNOT_NEGATE,
    );
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, sumtp, _, _, _| {
        !c.is_set_card(duel, &[SET_RYU_GE]) && sumtp & SUMMON_TYPE_PENDULUM == SUMMON_TYPE_PENDULUM
    });
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_CHAIN_SOLVING);
    e2.set_range(duel, LOCATION_PZONE);
    e2.set_condition(duel, discon);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_HAND);
    e3.set_count_limit(duel, 1, ID);
    e3.set_cost(duel, spcost);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_DESTROY);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e4.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e4.set_count_limit(duel, 1, (ID, 1));
    e4.set_target(duel, destg);
    e4.set_operation(duel, stock::destroy_first_target);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 2));
    e5.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e5.set_property(duel, EFFECT_FLAG_DELAY);
    e5.set_code(duel, EVENT_DESTROYED);
    e5.set_count_limit(duel, 1, (ID, 2));
    e5.set_target(duel, pltg);
    e5.set_operation(duel, plop);
    c.register_effect(duel, e5);
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    let ch = i32::from(duel.get_current_chain_full(true)) - 1;
    let Ok(ch) = u8::try_from(ch) else { return false };
    ch > 0
        && a.ep == 1 - tp
        && !duel.has_flag_effect(tp, ID)
        && duel.is_chain_disablable(a.ev as u8)
        && chain::is_triggering_player(duel, ch, tp)
        && chain::is_triggering_type(duel, ch, TYPE_SPELL)
        && chain::is_triggering_setcode(duel, ch, &[SET_RYU_GE])
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if duel.select_effect_yes_no_full(tp, c, aux::stringid(ID, 3))? {
        duel.hint(HINT_CARD, 0, ID.into());
        duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
        if duel.negate_effect(a.ev as u8) {
            duel.break_effect()?;
            duel.destroy(c, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn spcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RYU_GE]) && !c.is_code(duel, ID)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, spcostfilter, (1, 1), true, Some(&aux::release_check_mmz), Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RELEASE);
    let g = duel.select_release_group_cost(tp, spcostfilter, (1, 1), true, Some(&aux::release_check_mmz), Some(c.into()))?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    Ok(false)
}

fn plfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_continuous_spell_trap(duel)
        && c.is_set_card(duel, &[SET_RYU_GE])
        && !c.is_forbidden(duel)
        && c.check_unique_on_field(duel, tp)
        && (c.is_faceup(duel) || !c.is_location(duel, LOCATION_REMOVED))
}

fn pltg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let location = LOCATION_DECK | LOCATION_GRAVE | LOCATION_REMOVED;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_matching_card(|c, duel| plfilter(c, duel, tp), tp, location, 0, 1, None));
    }
    duel.set_possible_operation_info(0, CATEGORY_LEAVE_GRAVE, None, 1, tp, 0);
    Ok(false)
}

fn plop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.get_location_count(tp, LOCATION_SZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    let location = LOCATION_DECK | LOCATION_GRAVE | LOCATION_REMOVED;
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(|c, duel| plfilter(c, duel, tp)),
        tp,
        location,
        0,
        1,
        1,
        None,
    )?;
    if let Some(tc) = g.get_first() {
        duel.move_to_field(tc, tp, tp, LOCATION_SZONE, POS_FACEUP, true)?;
    }
    Ok(())
}
