// Golem that Guards the Millennium Treasures
use super::c38775407::add_placement_effects;
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 74169516;

pub const LISTED_NAMES: &[u32] = &[CARD_MILLENNIUM_CROSS, 63017368];
// In the hand to place and Special Summon, Millennium Ankh in the hand to reveal, and Wedju Temple in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (CARD_MILLENNIUM_CROSS, HAND), (63017368, DECK), (63017368, DECK)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_INACTIVATE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::Chain(efilter));
    c.register_effect(duel, e1);
    add_placement_effects::<ID>(duel, c);
}

fn efilter(duel: &Duel, e: EffectId, ct: u8) -> bool {
    let p = e.get_handler_player(duel);
    let tp: u8 = duel.get_chain_info(ct, CHAININFO_TRIGGERING_PLAYER);
    duel.get_chain_info::<Option<EffectId>>(ct, CHAININFO_TRIGGERING_EFFECT)
        .is_some_and(|te| p == tp && te.get_handler(duel).is_code(duel, CARD_MILLENNIUM_CROSS))
}
