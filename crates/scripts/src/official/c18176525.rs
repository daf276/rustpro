// Mitsurugi no Mikoto, Saji
use super::c40543231::register_destroy_replace;
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};
use engine::constants::MSG_SELECT_EFFECTYN;

const ID: u32 = 18176525;
const MITSURUGI_ARAMASA: u32 = 40543231;
const MITSURUGI_PRAYERS: u32 = 45171524;
const MITSURUGI_RITUAL: u32 = 81560239;

// Copies on the field to Tribute for the Reptile Aramasa's destruction by the opponent's Raigeki, in the hand to Summon, and
// Mitsurugi Spells in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, HAND),
    (MITSURUGI_ARAMASA, MZONE),
    (MITSURUGI_PRAYERS, DECK),
    (MITSURUGI_RITUAL, DECK),
    (12580477, HAND | OPPONENT),
];
// The destruction replacement is no chain link; its prompt is the MSG_SELECT_EFFECTYN, counted for any card.
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Message(MSG_SELECT_EFFECTYN)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = e1.clone_effect(duel);
    e3.set_code(duel, EVENT_RELEASE);
    c.register_effect(duel, e3);
    register_destroy_replace(duel, c, ID);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MITSURUGI]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
