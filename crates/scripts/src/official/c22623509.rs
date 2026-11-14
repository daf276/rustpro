// Verre, the Maid of Endymion
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};
use engine::constants::MSG_ATTACK;

const ID: u32 = 22623509;

pub const LISTED_SERIES: &[u16] = &[SET_WITCHCRAFTER];

const MAGICAL_DIMENSION: u32 = 28553439;

pub const LISTED_NAMES: &[u32] = &[CARD_REGULUS_THE_PRINCE_OF_ENDYMION, MAGICAL_DIMENSION];

// Copies in the hand, Pot of Greed in either hand for the Spell activation, Regulus on the field for the ATK gain, and Witchcrafter
// Creation in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (55144522, HAND),
    (55144522, HAND | OPPONENT),
    (96228804, MZONE),
    (57916305, DECK),
    (57916305, DECK),
    (57916305, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::Message(MSG_ATTACK),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| {
        let regulus = aux::faceup_filter(|c, duel| c.is_code(duel, CARD_REGULUS_THE_PRINCE_OF_ENDYMION));
        duel.is_existing_matching_card(regulus, a.e.get_handler_player(duel), LOCATION_ONFIELD | LOCATION_GRAVE, 0, 1, None)
    });
    e1.set_value(duel, 2300);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, (ID, 0));
    e2.set_condition(duel, |duel, _| {
        duel.get_custom_activity_count(ID, 0, ACTIVITY_CHAIN) > 0 || duel.get_custom_activity_count(ID, 1, ACTIVITY_CHAIN) > 0
    });
    e2.set_target(duel, stock::special_summon_self_target);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
    duel.add_custom_activity_counter(ID, ACTIVITY_CHAIN, CounterFilter::Chain(|duel, re, _, _| !re.is_spell_effect(duel)));
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    ((c.is_set_card(duel, &[SET_WITCHCRAFTER]) && c.is_spell_trap(duel)) || c.is_code(duel, MAGICAL_DIMENSION)) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
