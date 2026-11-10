// Elemental Burst
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 61411502;
const ATTRIBUTES: u32 = ATTRIBUTE_EARTH | ATTRIBUTE_WATER | ATTRIBUTE_FIRE | ATTRIBUTE_WIND;

// In the hand with an EARTH, WATER, FIRE and WIND Normal Monster (Kojikocy, Island Turtle, Darkfire Soldier #1, Flying Kamakiri #2)
// to Tribute, and monsters on the opponent's side to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (1184620, MZONE),
    (4042268, MZONE),
    (5388481, MZONE),
    (3134241, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn spcheck(duel: &Duel, sg: CardSet, _: u8, _: CardSet) -> (bool, bool) {
    (
        aux::check_different_property_binary(duel, sg, &|c, duel| u64::from(c.get_attribute(duel) & ATTRIBUTES)),
        false,
    )
}

fn attribute(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTES)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, attribute, (4, 4), false, Some(&spcheck), None));
    }
    let sg = duel.select_release_group_cost(tp, attribute, (4, 4), false, Some(&spcheck), None)?;
    duel.release(sg, REASON_COST)?;
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    let g = duel.get_matching_group(aux::TRUE, tp, 0, LOCATION_ONFIELD, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, tp, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(aux::TRUE, a.tp, 0, LOCATION_ONFIELD, None);
    duel.destroy(g, REASON_EFFECT)?;
    Ok(())
}
