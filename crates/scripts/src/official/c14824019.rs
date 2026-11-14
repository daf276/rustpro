// Spellbook Magician of Prophecy
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 14824019;

pub const LISTED_SERIES: &[u16] = &[SET_SPELLBOOK];
// In the hand to Normal Summon, with Spellbook Spells in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (23314220, DECK), (23314220, DECK), (89739383, DECK), (96677818, DECK)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_target(duel, stock::operation_info_target::<CATEGORY_TOHAND, LOCATION_DECK>);
    e1.set_operation(duel, shop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_FLIP);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SPELLBOOK]) && c.is_spell(duel) && c.is_able_to_hand(duel)
}

fn shop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, filter, LOCATION_DECK)
}
