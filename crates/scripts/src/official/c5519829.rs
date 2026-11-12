// Counselor Lily
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 5519829;

// On the field as the Level 3 Tuner with Swordsman of Landstar (Level 3) for Iron Chain Dragon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (3573512, MZONE),
    (3573512, MZONE),
    (19974580, EXTRA),
    (19974580, EXTRA),
    (19974580, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BE_MATERIAL);
    e1.set_condition(duel, con);
    e1.set_cost(duel, Cost::pay_lp::<500>);
    e1.set_operation(duel, op);
    c.register_effect(duel, e1);
}

fn con(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_location(duel, LOCATION_GRAVE) && a.r == REASON_SYNCHRO
}

fn op(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(sync) = c.get_reason_card(duel) else { return Ok(()) };
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, 1000);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    sync.register_effect(duel, e1);
    Ok(())
}
