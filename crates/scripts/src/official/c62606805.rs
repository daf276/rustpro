// Mind Procedure
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 62606805;

pub const LISTED_SERIES: &[u16] = &[SET_TELEPORT];
// Copies in the hand to Normal Summon and on the field and in the Graveyard to be banished by the opponent's Night's End Sorcerer
// revived with Monster Reborn, with Emergency Teleport (a Teleport Quick-Play Spell) in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (ID, GRAVE),
    (67723438, DECK),
    (67723438, DECK),
    (36107810, GRAVE | OPPONENT),
    (36107810, GRAVE | OPPONENT),
    (83764718, HAND | OPPONENT),
    (83764718, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_NONTUNER);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::Card(|duel, _, sc| i32::from(sc.is_race(duel, RACE_PSYCHIC))));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, Cost::pay_lp::<2000>);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_condition(duel, |duel, a| a.re.is_some_and(|re| re.is_monster_effect(duel)));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_DELAY);
    e4.set_code(duel, EVENT_REMOVE);
    e4.set_count_limit(duel, 1, ID);
    e4.set_cost(duel, Cost::pay_lp::<1000>);
    e4.set_target(duel, stock::special_summon_self_target);
    e4.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e4);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_TELEPORT]) && (c.is_normal_spell(duel) || c.is_quick_play_spell(duel)) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK | LOCATION_GRAVE, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, aux::necro_valley_filter(thfilter), LOCATION_DECK | LOCATION_GRAVE)
}
