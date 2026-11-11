// Card Trooper
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 85087012;

// Copies on the field to mill for the ATK gain, and Dark Hole in the opponent's hand to destroy one and draw.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (53129443, HAND | OPPONENT)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, cost);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_condition(duel, drcon);
    e2.set_target(duel, drtg);
    e2.set_operation(duel, stock::draw);
    c.register_effect(duel, e2);
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_player_can_discard_deck_as_cost(tp, 1));
    }
    let ct: Vec<u64> = (1..=3)
        .rev()
        .filter(|&i| duel.is_player_can_discard_deck_as_cost(tp, i as usize))
        .collect();
    if let [only] = ct[..] {
        duel.discard_deck(tp, only as u16, REASON_COST)?;
        e.set_label(duel, 1);
    } else {
        duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
        let ac = duel.announce_number(tp, &ct)?;
        duel.discard_deck(tp, ac as u16, REASON_COST)?;
        e.set_label(duel, ac as i64);
    }
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, e) {
        let ct = e.get_label(duel) as i32;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END);
        e1.set_value(duel, ct * 500);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn drcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    a.r & REASON_DESTROY != 0 && c.is_previous_controler(duel, a.tp) && c.is_previous_location(duel, LOCATION_ONFIELD)
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(a.tp);
    duel.set_target_param(1);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, a.tp, 1);
    Ok(false)
}
