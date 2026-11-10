// Krebons
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 59575539;

// Copies on the field to be attacked by the opponent's Elfnote Lucina.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_code(duel, EVENT_BE_BATTLE_TARGET);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_cost(duel, Cost::pay_lp::<800>);
    e1.set_operation(duel, |duel, _| {
        duel.negate_attack()?;
        Ok(())
    });
    c.register_effect(duel, e1);
}
