// Ceruli, Guru of Dark World
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND};

const ID: u32 = 7623640;

// The Gates of Dark World on the field discards this card by its effect, banishing Zure, Knight of Dark World for its cost.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (ID, HAND), (33017655, 0), (7459013, GRAVE)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_HANDES);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_condition(duel, |duel, a| {
        a.re.is_some_and(|re| re.get_handler(duel).is_set_card(duel, &[SET_DARK_WORLD]))
    });
    e2.set_target(duel, hdtg);
    e2.set_operation(duel, hdop);
    c.register_effect(duel, e2);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_previous_location(duel, LOCATION_HAND)
        && a.r & (REASON_DISCARD | REASON_EFFECT) == REASON_DISCARD | REASON_EFFECT
}

pub fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let c = a.e.get_handler(duel);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.special_summon(c, 0, a.tp, 1 - a.tp, false, false, POS_FACEUP_DEFENSE)?;
    }
    Ok(())
}

fn hdtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, 1 - a.tp, 1);
    Ok(false)
}

fn hdop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.discard_hand(1 - a.tp, aux::TRUE, 1, 1, REASON_EFFECT | REASON_DISCARD)?;
    Ok(())
}
