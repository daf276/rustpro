// Assault Lich
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 86098176;

pub const LISTED_NAMES: &[u32] = &[CARD_ASSAULT_MODE, 40012727, 76407432];
// In the hand and on the field, with Crimson Blader/Assault Mode and a Level 4 or lower Zombie (The 13th Grave) in the Graveyard,
// Assault Slash and Assault Counter in the Deck, and Dark Plant with a Level 5 Synchro (Catastor, Accel Synchron) in the Extra Deck
// to use it as material.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (ID, MZONE),
    (50604072, GRAVE),
    (32864, GRAVE),
    (40012727, DECK),
    (76407432, DECK),
    (13193642, MZONE),
    (13193642, MZONE),
    (26593852, EXTRA),
    (26593852, EXTRA),
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
    c.register_effect(duel, e1);
    let e2a = duel.create_effect(c);
    e2a.set_description(duel, aux::stringid(ID, 0));
    e2a.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2a.set_property(duel, EFFECT_FLAG_DELAY);
    e2a.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2a.set_count_limit(duel, 1, ID);
    e2a.set_target(duel, sptg);
    e2a.set_operation(duel, spop);
    c.register_effect(duel, e2a);
    let e2b = e2a.clone_effect(duel);
    e2b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2b);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_BE_MATERIAL);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_location(duel, LOCATION_GRAVE) && a.r == REASON_SYNCHRO
    });
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    (c.lists_code(duel, CARD_ASSAULT_MODE) || (c.is_level_below(duel, 4) && c.is_race(duel, RACE_ZOMBIE)))
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_GRAVE)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_GRAVE, POS_FACEUP)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_code(duel, 40012727) || c.is_code(duel, 76407432)) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
