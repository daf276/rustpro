// Full-Speed Warrior
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 17201951;

pub const LISTED_SERIES: &[u16] = &[SET_WARRIOR];
pub const LISTED_NAMES: &[u32] = &[CARD_JUNK_SYNCHRON, CARD_JUNK_WARRIOR];

// In the hand to Normal Summon, with Junk Synchron in the Deck to search, and Assault Sonic Warrior and a Synchro
// Junk Warrior to boost.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (CARD_JUNK_SYNCHRON, DECK),
    (CARD_JUNK_SYNCHRON, DECK),
    (18711696, MZONE),
    (CARD_JUNK_WARRIOR, MZONE),
    (CARD_JUNK_WARRIOR, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1a = duel.create_effect(c);
    e1a.set_description(duel, aux::stringid(ID, 0));
    e1a.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1a.set_property(duel, EFFECT_FLAG_DELAY);
    e1a.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1a.set_count_limit(duel, 1, ID);
    e1a.set_target(duel, thtg);
    e1a.set_operation(duel, thop);
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e1b);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_condition(duel, |duel, a| {
        duel.is_battle_phase() && duel.is_turn_player(a.e.get_handler_player(duel))
    });
    e2.set_target_card(duel, |duel, _, c| {
        c.lists_code(duel, CARD_JUNK_WARRIOR) || (c.is_type(duel, TYPE_SYNCHRO) && c.is_original_set_card(duel, &[SET_WARRIOR]))
    });
    e2.set_value(duel, 900);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_code(duel, CARD_JUNK_SYNCHRON) || (c.is_spell_trap(duel) && c.lists_code(duel, CARD_JUNK_WARRIOR))) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
