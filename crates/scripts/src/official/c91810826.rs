// Tenpai Dragon Chundra
use super::c39931513::{synchop, synchtg};
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 91810826;

pub const LISTED_NAMES: &[u32] = &[ID];

// Tenpai Dragon Paidra, a Level 3 FIRE Dragon, on the field to Synchro Summon with Ancient Fairy Dragon in the Battle Phase and
// to Special Summon from the Deck, and a copy in the hand to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, HAND),
    (39931513, MZONE),
    (39931513, DECK),
    (39931513, DECK),
    (13597785, MZONE | OPPONENT),
    (25862681, EXTRA),
    (25862681, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_hint_timing(duel, 0, TIMING_BATTLE_START | TIMING_BATTLE_END);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, |duel, _| duel.is_battle_phase());
    e1.set_target(duel, synchtg);
    e1.set_operation(duel, synchop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, selfspcon);
    e2.set_target(duel, stock::special_summon_self_target_player_0);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_BATTLE_START);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, decksptg);
    e3.set_operation(duel, deckspop);
    c.register_effect(duel, e3);
}

fn selfspconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_attribute(duel, ATTRIBUTE_FIRE) && c.is_race(duel, RACE_DRAGON)
}

fn selfspcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(selfspconfilter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn deckspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_below(duel, 4)
        && c.is_attribute(duel, ATTRIBUTE_FIRE)
        && c.is_race(duel, RACE_DRAGON)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && !c.is_code(duel, ID)
}

fn decksptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| deckspfilter(c, duel, a.e, a.tp), LOCATION_DECK)
}

fn deckspop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| deckspfilter(c, duel, a.e, a.tp), LOCATION_DECK, POS_FACEUP)
}
