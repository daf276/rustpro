// Ryu-Ge Realm - Dino Domains
use super::c55154344::{activation, efftg, grant, is_realm_cost, to_deck_cost};
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 82661630;

pub const LISTED_SERIES: &[u16] = &[SET_RYU_GE];
pub const LISTED_NAMES: &[u32] = &[ID];
// Copies in the hand to activate, with Kyoro Ryu-Ge Kaiva (Level 10 Dinosaur) on the field to be granted the effect.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (93509766, MZONE), (93509766, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesGranted(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    activation::<ID>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_RYU_GE]));
    e1.set_value(duel, 300);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_NEGATE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, negcon);
    e2.set_cost(duel, |duel, a, chk| to_deck_cost(duel, a, chk, is_realm_cost));
    e2.set_target(duel, negtg);
    e2.set_operation(duel, |duel, a| {
        duel.negate_activation(a.ev as u8);
        Ok(())
    });
    grant(duel, c, e2, efftg::<RACE_DINOSAUR>);
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    let trig_loc: u32 = duel.get_chain_info(0, CHAININFO_TRIGGERING_LOCATION);
    let trig_typ: u32 = duel.get_chain_info(0, CHAININFO_TRIGGERING_TYPE);
    let trig_atk: i64 = duel.get_chain_info(0, CHAININFO_TRIGGERING_ATTACK);
    trig_loc & LOCATION_MZONE > 0 && trig_typ & TYPE_MONSTER > 0 && i64::from(a.e.get_handler(duel).get_attack(duel)) > trig_atk
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, a.tp, 0);
    Ok(false)
}
