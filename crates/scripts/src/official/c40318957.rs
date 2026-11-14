// Performapal Skullcrobat Joker
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, PZONE};

const ID: u32 = 40318957;

pub const LISTED_SERIES: &[u16] = &[SET_PERFORMAPAL, SET_MAGICIAN, SET_ODD_EYES];
pub const LISTED_NAMES: &[u32] = &[ID];

// In the Pendulum Zone and the hand to Normal Summon, with Supreme King Gate Magician (a Pendulum "Magician") in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (14513273, DECK),
    (14513273, DECK),
    (14513273, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_range(duel, LOCATION_PZONE);
    e2.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e2.set_property(
        duel,
        EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CANNOT_NEGATE,
    );
    e2.set_target_range(duel, 1, 0);
    e2.set_target_spsummon(duel, |duel, _, c, _, sumtype, _, _, _| {
        !filter(c, duel) && sumtype & SUMMON_TYPE_PENDULUM == SUMMON_TYPE_PENDULUM
    });
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_SUMMON_SUCCESS);
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_PERFORMAPAL])
        || (c.is_set_card(duel, &[SET_MAGICIAN]) && c.is_type(duel, TYPE_PENDULUM))
        || c.is_set_card(duel, &[SET_ODD_EYES])
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    filter(c, duel) && !c.is_code(duel, ID) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
