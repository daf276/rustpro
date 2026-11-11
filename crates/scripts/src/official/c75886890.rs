// Vylon Sphere
use super::c1281505::add_equip_trigger;
use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE, OPPONENT};

const ID: u32 = 75886890;

// Two copies and Sangans on the field with Black Pendants in the Graveyard, and Elfnote Lucinas on the opponent's side to
// attack, so a Sphere is destroyed, equipped to a Sangan, and then sent to the Graveyard to equip a Pendant.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (26202165, MZONE),
    (26202165, MZONE),
    (65169794, GRAVE),
    (65169794, GRAVE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    add_equip_trigger(duel, c, ID, eqtg);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_category(duel, CATEGORY_EQUIP);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_cost(duel, eqcost2);
    e2.set_target(duel, eqtg2);
    e2.set_operation(duel, eqop2);
    c.register_effect(duel, e2);
}

fn eqfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel)
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0 && duel.is_existing_target(eqfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    duel.select_target(tp, eqfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    Ok(false)
}

fn eqcost2(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_grave_as_cost(duel));
    }
    a.e.set_label_object_group(duel, c.get_equip_target(duel).map(CardSet::from));
    duel.send_to_grave(c, REASON_COST)?;
    Ok(false)
}

fn filter2(c: CardId, duel: &Duel, ec: CardId) -> bool {
    c.is_type(duel, TYPE_EQUIP) && c.check_equip_target(duel, ec)
}

fn eqtg2(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let ec = e.get_handler(duel).get_equip_target(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && ec.is_some_and(|ec| filter2(chkc, duel, ec)));
    }
    let ec_label = e.get_label_object_group(duel).and_then(|g| g.get_first());
    if chk == 0 {
        return Ok(ec.is_some_and(|ec| duel.is_existing_target(|c, duel| filter2(c, duel, ec), tp, LOCATION_GRAVE, 0, 1, None)));
    }
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
    if let Some(ec) = ec_label {
        duel.select_target(tp, |c, duel| filter2(c, duel, ec), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
        ec.create_effect_relation(duel, e);
    }
    Ok(false)
}

fn eqop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && let Some(ec) = e.get_label_object_group(duel).and_then(|g| g.get_first())
        && tc.is_relate_to_effect(duel, e)
    {
        duel.equip(tp, tc, ec)?;
    }
    Ok(())
}
