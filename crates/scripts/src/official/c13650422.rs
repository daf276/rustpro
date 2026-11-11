// Evil HERO Adusted Gold
use crate::prelude::*;
use crate::testing::{DECK, HAND, MZONE};

const ID: u32 = 13650422;

pub const LISTED_NAMES: &[u32] = &[CARD_DARK_FUSION];

// In the hand and on the field, with Dark Fusion in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, MZONE), (CARD_DARK_FUSION, DECK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::self_discard);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_CANNOT_ATTACK);
    e2.set_condition(duel, atkcon);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_hand(duel) && !c.is_code(duel, ID) && (c.is_code(duel, CARD_DARK_FUSION) || c.lists_code(duel, CARD_DARK_FUSION))
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    let filter = aux::faceup_filter(|c, duel| c.is_type(duel, TYPE_FUSION));
    !duel.is_existing_matching_card(filter, a.e.get_handler_player(duel), LOCATION_MZONE, 0, 1, None)
}
