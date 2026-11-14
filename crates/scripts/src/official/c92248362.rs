// K9-17 Izuna
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 92248362;

pub const LISTED_SERIES: &[u16] = &[SET_K9];
pub const LISTED_NAMES: &[u32] = &[ID];

// Copies in the hand to Summon, a K9 monster in the Deck to send, and an opponent's Elfnote Lucina in the hand and Graveyard
// to activate from there.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (91025875, DECK),
    (91025875, DECK),
    (13597785, HAND | OPPONENT),
    (13597785, GRAVE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::TributeSummoned(0),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SUMMON_PROC);
    e1.set_condition(duel, ntcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| {
        duel.is_main_phase() && duel.get_custom_activity_count(ID, 1 - a.tp, ACTIVITY_CHAIN) > 0
    });
    e2.set_target(duel, sptg);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e2);
    duel.add_custom_activity_counter(ID, ACTIVITY_CHAIN, CounterFilter::Chain(chainfilter));
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_TOGRAVE);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_SUMMON_SUCCESS);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, tgtg);
    e3.set_operation(duel, tgop);
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e4);
}

// Also the filter of K9-00 Lupis.
pub fn chainfilter(duel: &Duel, re: EffectId, _: u8, _: u8) -> bool {
    !(re.is_monster_effect(duel) && re.get_activate_location(duel) & (LOCATION_HAND | LOCATION_GRAVE) > 0)
}

pub fn ntcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    a.min == 0
        && c.is_level_above(duel, 5)
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.get_field_group_count(tp, 0, LOCATION_HAND) >= 2
}

pub fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_K9]) && c.is_able_to_grave(duel) && !c.is_code(duel, ID)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, tgfilter, LOCATION_DECK, CATEGORY_TOGRAVE)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::send_to_grave(duel, a, tgfilter, LOCATION_DECK)
}
