// Geomancer of the Ice Barrier
use crate::prelude::*;
use crate::testing::{HAND, MZONE};

const ID: u32 = 56704140;

// On the field with a card in the hand to discard.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (15025844, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_NO_TURN_RESET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

pub fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(aux::ice_barrier_discard_cost_check(duel, tp, aux::TRUE, true, 1));
    }
    aux::ice_barrier_discard_cost(duel, tp, aux::TRUE, true, (1, 1))?;
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let attr = duel.announce_attribute(a.tp, 1, ATTRIBUTE_ALL)?;
    a.e.set_label(duel, i64::from(attr));
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if c.is_facedown(duel) || !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let attr = e.get_label(duel);
    c.set_hint(duel, CHINT_ATTRIBUTE, attr as u64);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EFFECT_CANNOT_BE_BATTLE_TARGET);
    e1.set_label(duel, attr);
    e1.set_value(duel, Value::Card(tgval));
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    c.register_effect(duel, e1);
    Ok(())
}

fn tgval(duel: &Duel, e: EffectId, c: CardId) -> i32 {
    i32::from(c.is_attribute(duel, e.get_label(duel) as u32) && !c.is_immune_to_effect(duel, e))
}
