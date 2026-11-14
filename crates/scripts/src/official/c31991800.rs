// Majespecter Raccoon - Bunbuku
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 31991800;

pub const LISTED_SERIES: &[u16] = &[SET_MAJESPECTER];
// In the hand to Normal Summon, with copies in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (ID, DECK), (ID, DECK)];
pub const CHECKS: &[Check] = &[Check::Resolves];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE);
    e4.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e4.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_SINGLE);
    e5.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e5.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e5.set_range(duel, LOCATION_MZONE);
    e5.set_value(duel, Value::EffectPlayer(|duel, e, _, tp| tp != e.get_handler_player(duel)));
    c.register_effect(duel, e5);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MAJESPECTER]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
