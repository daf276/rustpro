// Big Eye
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 16768387;

// Three copies on the field to be flipped by the opponent's attackers.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (14851496, MZONE | OPPONENT),
    (70781052, MZONE | OPPONENT),
    (48766543, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_FLIP);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let ct = 5.min(duel.get_field_group_count(tp, LOCATION_DECK, 0));
    if ct == 0 {
        return Ok(());
    }
    if ct != 1 {
        duel.announce_number_range(tp, 1, ct as u32)?;
    }
    duel.sort_decktop(tp, tp, ct as u16)?;
    Ok(())
}
