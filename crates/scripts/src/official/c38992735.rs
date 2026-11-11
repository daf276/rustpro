// Wave-Motion Cannon
use crate::prelude::*;
use crate::testing::{Check, HAND, SZONE};

const ID: u32 = 38992735;

// One in the hand to activate and one already in the Spell & Trap Zone, to count the turns of.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, SZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, reset);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DAMAGE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_cost(duel, Cost::self_to_grave);
    e2.set_target(duel, tg);
    e2.set_operation(duel, stock::damage);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e3.set_code(duel, EVENT_PHASE_START | PHASE_STANDBY);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_operation(duel, turncount);
    c.register_effect(duel, e3);
}

fn turncount(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.is_turn_player(1 - a.tp) {
        return Ok(());
    }
    let c = a.e.get_handler(duel);
    let ct = c.get_turn_counter(duel) + 1;
    c.set_turn_counter(duel, ct);
    Ok(())
}

fn reset(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    a.e.get_handler(duel).set_turn_counter(duel, 0);
    Ok(false)
}

fn tg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    if chk == 0 {
        return Ok(c.get_turn_counter(duel) > 0);
    }
    let dam = u32::from(c.get_turn_counter(duel)) * 1000;
    duel.set_target_player(1 - tp);
    duel.set_target_param(dam);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, dam as i32);
    Ok(false)
}
