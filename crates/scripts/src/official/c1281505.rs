// Vylon Tetra
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 1281505;

// Copies and Sangans on the field, with Elfnote Lucinas on the opponent's side to attack, so a Tetra is destroyed and
// equipped to a Sangan, and Dark Holes in the hand to destroy the Sangan later.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (26202165, MZONE),
    (26202165, MZONE),
    (26202165, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (53129443, HAND),
    (53129443, HAND),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Replaced];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    add_equip_trigger(duel, c, ID, eqtg);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_EQUIP + EFFECT_TYPE_CONTINUOUS);
    e3.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e3.set_code(duel, EFFECT_DESTROY_REPLACE);
    e3.set_target(duel, reptg);
    e3.set_operation(duel, repop);
    c.register_effect(duel, e3);
}

// The trigger the four Vylon equip monsters share, which differs only in its target.
pub(super) fn add_equip_trigger(duel: &mut Duel, c: CardId, id: u32, target: fn(&mut Duel, &Args, i32, Option<CardId>) -> Script<bool>) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(id, 0));
    e1.set_category(duel, CATEGORY_EQUIP);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_condition(duel, eqcon);
    e1.set_cost(duel, Cost::pay_lp::<500>);
    e1.set_target(duel, target);
    e1.set_operation(duel, eqop);
    c.register_effect(duel, e1);
}

fn eqcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).get_previous_location(duel) == LOCATION_MZONE
}

fn eqfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel)
}

pub(super) fn eqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0 && duel.is_existing_target(eqfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    duel.select_target(tp, eqfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    let c = a.e.get_handler(duel);
    duel.set_operation_info(0, CATEGORY_EQUIP, Some(c.into()), 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(c.into()), 1, 0, 0);
    Ok(false)
}

pub(super) fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && c.is_relate_to_effect(duel, e)
        && tc.is_faceup(duel)
        && tc.is_relate_to_effect(duel, e)
    {
        duel.equip(tp, c, tc)?;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_EQUIP_LIMIT);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_value(duel, Value::Card(eqlimit));
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn eqlimit(duel: &Duel, e: EffectId, c: CardId) -> i32 {
    i32::from(c.is_controler(duel, e.get_handler_player(duel)))
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(!c.is_reason(duel, REASON_REPLACE));
    }
    duel.select_effect_yes_no_full(a.tp, c, 96)
}

fn repop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.destroy(a.e.get_handler(duel), REASON_EFFECT | REASON_REPLACE)?;
    Ok(())
}
