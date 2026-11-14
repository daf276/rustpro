// Scrap Goblin
use super::c19139516::{desop, thcon, thfilter};
use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE, OPPONENT};

const ID: u32 = 83135907;

pub const LISTED_SERIES: &[u16] = &[SET_SCRAP];
pub const LISTED_NAMES: &[u32] = &[ID];
// On the field in Defense Position with Scrap Raptor to destroy it, and Scrap monsters in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (80529459, MZONE),
    (16449363, GRAVE),
    (4334811, GRAVE),
    (4334811, GRAVE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2a = duel.create_effect(c);
    e2a.set_description(duel, aux::stringid(ID, 0));
    e2a.set_category(duel, CATEGORY_DESTROY);
    e2a.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2a.set_code(duel, EVENT_PHASE | PHASE_BATTLE);
    e2a.set_range(duel, LOCATION_MZONE);
    e2a.set_count_limit(duel, 1, 0);
    e2a.set_target(duel, destg);
    e2a.set_operation(duel, desop);
    c.register_effect(duel, e2a);
    let e2b = duel.create_effect(c);
    e2b.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2b.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2b.set_code(duel, EVENT_BE_BATTLE_TARGET);
    e2b.set_operation(duel, regop);
    c.register_effect(duel, e2b);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_condition(duel, thcon);
    e3.set_target(duel, thtg);
    e3.set_operation(duel, stock::first_target_to_hand);
    c.register_effect(duel, e3);
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.get_flag_effect(duel, ID) > 0);
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_position(duel, POS_FACEUP_DEFENSE) {
        c.register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_BATTLE, 0, 1);
    }
    Ok(())
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && thfilter::<ID>(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter::<ID>, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thfilter::<ID>, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, tp, 0);
    Ok(false)
}
