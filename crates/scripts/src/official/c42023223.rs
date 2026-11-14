// Alpha The Electromagnet Warrior
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 42023223;

pub const LISTED_SERIES: &[u16] = &[SET_MAGNA_WARRIOR, SET_MAGNET_WARRIOR];

// Valkyrion the Magna Warrior to search and Alpha The Magnet Warrior (Level 4) to summon from the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (75347539, DECK),
    (75347539, DECK),
    (99785935, DECK),
    (99785935, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_TRIGGER_O + EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    register_tribute_summon::<ID>(duel, c);
}

// The Quick Effect Alpha and Gamma share: tribute it on the opponent's turn to summon a Level 4 "Magnet Warrior" from the Deck.
pub fn register_tribute_summon<const CODE: u32>(duel: &mut Duel, c: CardId) {
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(CODE, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_hint_timing(duel, 0, TIMING_END_PHASE);
    e3.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e3.set_cost(duel, Cost::self_tribute);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MAGNA_WARRIOR]) && c.is_level(duel, &[8]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn spfilter(e: EffectId, tp: u8) -> impl Fn(CardId, &Duel) -> bool {
    move |c, duel| {
        c.is_set_card(duel, &[SET_MAGNET_WARRIOR]) && c.is_level(duel, &[4]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
    }
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let mut ft = duel.get_location_count(a.tp, LOCATION_MZONE);
    if a.e.get_handler(duel).get_sequence(duel) < 5 {
        ft += 1;
    }
    if chk == 0 {
        return Ok(ft > 0 && duel.is_existing_matching_card(spfilter(a.e, a.tp), a.tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, spfilter(a.e, a.tp), LOCATION_DECK, POS_FACEUP)
}
