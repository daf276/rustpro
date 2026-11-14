// Dark Beckoning Beast
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 81034083;

pub const LISTED_NAMES: &[u32] = &[69890967, 6007213, 32491822, ID];

// Copies in hand to be Normal Summoned, the Sacred Beasts in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (69890967, DECK),
    (6007213, DECK),
    (32491822, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_EXTRA_SUMMON_COUNT);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_HAND | LOCATION_MZONE, 0);
    e2.set_target_card(duel, |duel, _, c| {
        c.is_race(duel, RACE_FIEND) && c.is_attack(duel, &[0]) && c.is_defense(duel, &[0])
    });
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    [69890967, 6007213, 32491822]
        .into_iter()
        .any(|code| c.is_code(duel, code) || c.lists_code(duel, code))
        && !c.is_code(duel, ID)
        && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
