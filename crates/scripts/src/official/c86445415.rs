// Red Gadget
use super::c41172955::{op, tg};
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 86445415;
const YELLOW_GADGET: u32 = 13839120;

pub const LISTED_NAMES: &[u32] = &[YELLOW_GADGET];

// Four copies in the hand to Normal Summon, Yellow Gadget in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (YELLOW_GADGET, DECK),
    (YELLOW_GADGET, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e1.set_type(duel, EFFECT_TYPE_TRIGGER_O + EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_target(duel, tg::<YELLOW_GADGET>);
    e1.set_operation(duel, op::<YELLOW_GADGET>);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
}
