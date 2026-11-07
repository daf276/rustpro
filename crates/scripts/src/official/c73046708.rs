// Armored Xyz
use crate::prelude::*;
use crate::testing::{GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 73046708;

// Tornado Dragon on the field and Norito the Moral Leader in the GY to equip, and opposing monsters to attack twice.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (6983839, MZONE),
    (14152862, GRAVE),
    (7359741, MZONE | OPPONENT),
    (10992251, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_EQUIP);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn eqfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_XYZ) && !c.is_forbidden(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let mut ft = duel.get_location_count(tp, LOCATION_SZONE);
    if e.get_handler(duel).is_location(duel, LOCATION_HAND) {
        ft -= 1;
    }
    if chk == 0 {
        return Ok(ft > 0
            && duel.is_existing_target(CardId::is_faceup, tp, LOCATION_MZONE, 0, 1, None)
            && duel.is_existing_target(eqfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, CardId::is_faceup, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    let eqc = duel.select_target(tp, eqfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_EQUIP, eqc, 1, tp, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_SZONE) <= 0 {
        return Ok(());
    }
    let g = duel.get_target_cards(e).unwrap_or_default();
    if g.len() != 2 {
        return Ok(());
    }
    let (Some(tc), Some(eqc)) = (
        g.filter(duel, |c, duel| c.is_location(duel, LOCATION_MZONE), None).get_first(),
        g.filter(duel, |c, duel| c.is_location(duel, LOCATION_GRAVE), None).get_first(),
    ) else {
        return Ok(());
    };
    if !tc.equip_by_effect_and_limit_register(duel, e, tp, eqc, None, Some(true))? {
        return Ok(());
    }
    let (atk, att) = (eqc.get_base_attack(duel), eqc.get_attribute(duel));
    let e0 = duel.create_effect(eqc);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e0.set_code(duel, EFFECT_EQUIP_LIMIT);
    e0.set_value(duel, Value::Card(super::c38423248::equip_limit));
    e0.set_label_object_group(duel, Some(tc.into()));
    e0.set_reset(duel, RESET_EVENT + RESETS_STANDARD);
    eqc.register_effect(duel, e0);
    let e1 = duel.create_effect(eqc);
    e1.set_type(duel, EFFECT_TYPE_EQUIP);
    e1.set_code(duel, EFFECT_SET_ATTACK);
    e1.set_value(duel, atk);
    e1.set_reset(duel, RESET_EVENT + RESETS_STANDARD);
    eqc.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_ADD_ATTRIBUTE);
    e2.set_value(duel, att as i32);
    eqc.register_effect(duel, e2);
    let e3 = duel.create_effect(eqc);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_DAMAGE_STEP_END);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_condition(duel, atkcon);
    e3.set_cost(duel, Cost::self_to_grave);
    e3.set_operation(duel, |duel, _| {
        duel.chain_attack();
        Ok(())
    });
    e3.set_reset(duel, RESET_EVENT + RESETS_STANDARD);
    eqc.register_effect(duel, e3);
    Ok(())
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    let ec = a.e.get_handler(duel).get_equip_target(duel);
    ec == duel.get_attacker() && ec.is_some_and(|ec| duel.check_mut(|duel| ec.can_chain_attack(duel)))
}
