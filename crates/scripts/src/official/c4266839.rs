// Kiseitai
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 4266839;

// Face-down on the field, to be attacked by the opponent's Elfnote Lucinas.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_EQUIP);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_BATTLE_CONFIRM);
    e1.set_condition(duel, eqcon);
    e1.set_operation(duel, eqop);
    c.register_effect(duel, e1);
}

fn eqcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    duel.get_attack_target() == Some(c) && c.get_battle_position(duel) == POS_FACEDOWN_DEFENSE
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let Some(tc) = duel.get_attacker() else { return Ok(()) };
    if !tc.is_relate_to_battle(duel) || !c.is_relate_to_battle(duel) {
        return Ok(());
    }
    if duel.get_location_count(tp, LOCATION_SZONE) <= 0 || tc.is_facedown(duel) {
        duel.destroy(c, REASON_EFFECT)?;
        return Ok(());
    }
    duel.equip(tp, c, tc)?;
    let e1 = duel.create_effect(tc);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_EQUIP_LIMIT);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    e1.set_value(duel, Value::Card(|duel, e, c| i32::from(e.get_owner(duel) == c)));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_RECOVER);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e2.set_target(duel, retg);
    e2.set_operation(duel, reop);
    e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    c.register_effect(duel, e2);
    Ok(())
}

fn retg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let ec = a.e.get_handler(duel).get_equip_target(duel);
    unported!(duel, ec.is_none());
    let half = ec.map_or(0, |ec| (f64::from(ec.get_attack(duel)) / 2.0).round() as i32);
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, a.tp, half);
    Ok(false)
}

fn reop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    if let Some(ec) = c.get_equip_target(duel) {
        let half = (f64::from(ec.get_attack(duel)) / 2.0).round() as u32;
        duel.recover(a.tp, half, REASON_EFFECT)?;
    }
    Ok(())
}
