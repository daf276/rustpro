// Koa'ki Meiru Doom
use engine::constants::{MSG_CARD_SELECTED, MSG_CHAIN_DISABLED};

use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 80925836;

pub const LISTED_NAMES: &[u32] = &[36623431];

// On the field with Iron Core of Koa'ki Meiru in the hand for its upkeep.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (36623431, HAND)];
// Both effects are continuous: the upkeep shows as the card's selection hint, the negation as a disabled chain link.
pub const CHECKS: &[Check] = &[Check::Message(MSG_CARD_SELECTED), Check::Message(MSG_CHAIN_DISABLED)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EVENT_PHASE + PHASE_END);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e1.set_operation(duel, |duel, a| super::c45041488::upkeep(duel, a, ID, RACE_FIEND, true));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_FIELD);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EVENT_CHAIN_ACTIVATING);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.is_main_phase()
        && let Some(re) = a.re
        && re.is_monster_effect(duel)
        && re.get_handler(duel).is_attribute(duel, ATTRIBUTE_DARK + ATTRIBUTE_LIGHT)
    {
        duel.negate_effect(a.ev as u8);
    }
    Ok(())
}
