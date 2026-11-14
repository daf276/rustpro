// Mitsurugi no Mikoto, Aramasa
use engine::constants::MSG_SELECT_EFFECTYN;

use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 40543231;

pub const LISTED_SERIES: &[u16] = &[SET_MITSURUGI];
pub const LISTED_NAMES: &[u32] = &[ID];

// Copies on the field to Tribute for each other's destruction by the opponent's Raigeki, and in the hand and Deck to
// Summon, and a Mitsurugi (Ame no Habakiri no Mitsurugi) in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, HAND),
    (13332685, DECK),
    (19899073, DECK),
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

// The destruction replacement the Mitsurugi Reptile monsters share, with the card's own `id` for its count limit.
pub fn register_destroy_replace(duel: &mut Duel, c: CardId, id: u32) {
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e4.set_code(duel, EFFECT_DESTROY_REPLACE);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, (id, 1));
    e4.set_target(duel, reptg);
    e4.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(repfilter(c, duel, e.get_handler_player(duel)))),
    );
    e4.set_operation(duel, |duel, a| {
        duel.release(a.e.get_handler(duel), REASON_EFFECT | REASON_REPLACE).map(drop)
    });
    c.register_effect(duel, e4);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MITSURUGI]) && c.is_monster(duel) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn repfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_controler(duel, tp)
        && c.is_location(duel, LOCATION_MZONE)
        && c.is_race(duel, RACE_REPTILE)
        && c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
        && !c.is_reason(duel, REASON_REPLACE)
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, c, tp) = (a.e, a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(a
            .eg
            .is_some_and(|eg| eg.is_exists(duel, |tc, duel| repfilter(tc, duel, tp), 1, Some(c.into())))
            && c.is_releasable_by_effect_full(duel, Some(e), None)
            && !c.is_status(duel, STATUS_DESTROY_CONFIRMED));
    }
    duel.select_effect_yes_no_full(tp, c, 96)
}
