// Rciela, Sinister Soul of the White Forest
use crate::prelude::*;
use crate::testing::{DECK, EXTRA, HAND, MZONE};

const ID: u32 = 77313225;

pub const LISTED_SERIES: &[u16] = &[SET_WHITE_FOREST];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// Two copies in the Extra Deck, Synchro Summoned from Silvy and Astellar of the White Forest on the field, with a Spell as its cost and
// Silvy in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (98385955, MZONE),
    (25592142, MZONE),
    (55144522, HAND),
    (98385955, DECK),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, thcost);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_target_card(duel, |duel, _, c| {
        c.is_race(duel, RACE_ILLUSION | RACE_SPELLCASTER) && c.is_type(duel, TYPE_SYNCHRO)
    });
    e2.set_value(duel, 500);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e3.set_value(duel, Value::EffectPlayer(aux::indoval));
    c.register_effect(duel, e3);
}

fn costfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel) && c.is_able_to_grave_as_cost(duel)
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::send_to_grave_cost(duel, a, chk, costfilter, LOCATION_HAND | LOCATION_ONFIELD)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_set_card(duel, &[SET_WHITE_FOREST]) || (c.is_attribute(duel, ATTRIBUTE_LIGHT) && c.is_race(duel, RACE_SPELLCASTER)))
        && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
