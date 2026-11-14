// Machina Gearframe
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 42940404;

pub const LISTED_SERIES: &[u16] = &[SET_MACHINA];

struct Procedure;

impl UnionProcedure for Procedure {
    fn f(c: CardId, duel: &Duel) -> bool {
        c.is_race(duel, RACE_MACHINE)
    }
}

pub const LISTED_NAMES: &[u32] = &[ID];

// In the hand and on the field with a Machine monster to equip (Mechanicalchaser), and Machina Ruinforce in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (7359741, MZONE),
    (46033517, DECK),
    (46033517, DECK),
    (46033517, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 2)), Check::ResolvesEffect(1068)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Union::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_TRIGGER_O + EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_target(duel, stg);
    e1.set_operation(duel, sop);
    c.register_effect(duel, e1);
}

fn sfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MACHINA]) && c.get_code(duel) != ID && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn stg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, sfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn sop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, sfilter, LOCATION_DECK)
}
