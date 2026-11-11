// Master Hyperion
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 55794644;

pub const LISTED_NAMES: &[u32] = &[CARD_SANCTUARY_SKY];
// In the hand, banishing The Agent of Creation - Venus, and on the field with Mystical Shine Ball in the Graveyard to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (64734921, HAND),
    (64734921, GRAVE),
    (39552864, GRAVE),
    (39552864, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, hspcon);
    e1.set_target(duel, hsptg);
    e1.set_operation(duel, hspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, condition);
    e2.set_cost(duel, cost);
    e2.set_target(duel, target);
    e2.set_operation(duel, stock::destroy_first_target);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_THE_AGENT]) && c.is_able_to_remove_as_cost(duel) && aux::sp_elim_filter(c, duel, true, true)
}

fn hspcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    let rg = duel.get_matching_group(spfilter, tp, LOCATION_HAND | LOCATION_ONFIELD | LOCATION_GRAVE, 0, None);
    aux::select_unselect_group_check(duel, rg, 1, 1, &aux::chkf_mmz(tp, 1))
}

fn hsptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let rg = duel.get_matching_group(spfilter, tp, LOCATION_HAND | LOCATION_ONFIELD | LOCATION_GRAVE, 0, None);
    let g = aux::select_unselect_group_full(duel, rg, 1, 1, &aux::chkf_mmz(tp, 1), tp, HINTMSG_REMOVE, None, true)?;
    if g.is_empty() {
        return Ok(false);
    }
    e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn hspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(g) = a.e.get_label_object_group(duel) else { return Ok(()) };
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(())
}

fn envfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_code(duel, CARD_SANCTUARY_SKY)
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let sanctuary = duel.is_existing_matching_card(envfilter, a.e.get_handler_player(duel), LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None)
        || duel.is_environment(CARD_SANCTUARY_SKY);
    c.get_flag_effect(duel, ID) < if sanctuary { 2 } else { 1 }
}

fn costfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_LIGHT) && c.is_race(duel, RACE_FAIRY) && c.is_able_to_remove_as_cost(duel)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::banish_cost(duel, a, chk, costfilter, LOCATION_GRAVE)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    a.e.get_handler(duel)
        .register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
    Ok(false)
}
