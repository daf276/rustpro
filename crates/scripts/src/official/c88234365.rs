// Darklord Tezcatlipoca
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 88234365;

pub const LISTED_SERIES: &[u16] = &[SET_DARKLORD];

const DARKLORD_GULGOLET: u32 = 84031359;
const DARKLORD_CONTACT: u32 = 14517422;
const RAIGEKI: u32 = 12580477;

// In the hand to replace the destruction of Darklord Gulgolet by Raigeki, and on the field with Darklord Contact in the
// GY to copy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (DARKLORD_GULGOLET, MZONE),
    (DARKLORD_GULGOLET, GRAVE),
    (DARKLORD_CONTACT, GRAVE),
    (RAIGEKI, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.set_sp_summon_once(duel, ID);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EFFECT_DESTROY_REPLACE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_target(duel, reptg);
    e1.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(repfilter(c, duel, e.get_handler_player(duel)))),
    );
    e1.set_operation(duel, repop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TODECK);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, Cost::pay_lp::<1000>);
    e2.set_target(duel, super::c52840267::cptg);
    e2.set_operation(duel, super::c52840267::cpop);
    c.register_effect(duel, e2);
}

fn repfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_set_card(duel, &[SET_DARKLORD])
        && c.is_location(duel, LOCATION_MZONE)
        && c.is_controler(duel, tp)
        && !c.is_reason(duel, REASON_REPLACE)
        && c.is_reason(duel, REASON_EFFECT | REASON_BATTLE)
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.is_discardable(duel) && a.eg.unwrap_or_default().is_exists(duel, |c, duel| repfilter(c, duel, tp), 1, None));
    }
    duel.select_effect_yes_no_full(tp, c, 96)
}

fn repop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.send_to_grave(a.e.get_handler(duel), REASON_EFFECT | REASON_DISCARD)?;
    Ok(())
}
