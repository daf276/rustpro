// Runick Allure
use crate::prelude::*;
use crate::testing::{Check, HAND};

const ID: u32 = 29595202;

pub const LISTED_NAMES: &[u32] = &[ID];

// Runick Quick-Play Spells to activate.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (31562086, HAND), (31562086, HAND), (67835547, HAND)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.set_unique_on_field(duel, 1, 0, |c, duel| c.is_code(duel, ID), LOCATION_ONFIELD);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_CHAIN_SOLVED);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_condition(duel, rmcon);
    e2.set_target(duel, rmtg);
    e2.set_operation(duel, rmop);
    c.register_effect(duel, e2);
}

fn rmcon(duel: &Duel, a: &Args) -> bool {
    a.re.is_some_and(|re| {
        re.is_spell_effect(duel) && re.get_handler(duel).is_type(duel, TYPE_QUICKPLAY) && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)
    })
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, 1 - a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_decktop_group(1 - a.tp, 1);
    if g.is_empty() {
        return Ok(());
    }
    duel.disable_shuffle_check(true);
    duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
    Ok(())
}
