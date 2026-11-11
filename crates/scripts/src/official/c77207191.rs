// Berfomet
use crate::prelude::*;
use crate::testing::{DECK, HAND, MZONE};

const ID: u32 = 77207191;
const GAZELLE: u32 = 5818798;

pub const LISTED_NAMES: &[u32] = &[GAZELLE];

// In the hand with a Fairy Normal Monster to tribute, and Gazelle the King of Mythical Beasts in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (38142739, MZONE), (GAZELLE, DECK), (GAZELLE, DECK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_TRIGGER_O + EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_target(duel, tg);
    e1.set_operation(duel, op);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_FLIP_SUMMON_SUCCESS);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, GAZELLE) && c.is_able_to_hand(duel)
}

fn tg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, filter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn op(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, filter, LOCATION_DECK)
}
