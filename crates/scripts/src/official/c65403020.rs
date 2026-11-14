// The End of Anubis
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};
use engine::constants::MSG_CHAIN_DISABLED;

const ID: u32 = 65403020;

// On the field, with Monster Reborn on both sides to target the GY.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (83764718, HAND),
    (83764718, HAND | OPPONENT),
    (83764718, HAND | OPPONENT),
];
// The negation is a continuous effect that forms no chain link.
pub const CHECKS: &[Check] = &[Check::Message(MSG_CHAIN_DISABLED)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EVENT_CHAIN_SOLVING);
    e1.set_condition(duel, condition);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    aux::double_snare_validity(duel, c, LOCATION_MZONE);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let loc: u32 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_LOCATION);
    if loc == LOCATION_GRAVE {
        return true;
    }
    if !a.re.is_some_and(|re| re.is_has_property(duel, EFFECT_FLAG_CARD_TARGET)) {
        return false;
    }
    let g: Option<CardSet> = duel.get_chain_info(a.ev as u8, CHAININFO_TARGET_CARDS);
    g.is_some_and(|g| g.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_GRAVE), 1, None))
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.negate_effect(a.ev as u8);
    Ok(())
}
