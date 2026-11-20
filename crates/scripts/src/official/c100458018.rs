// Raise Moon Wings Rapi
use super::c100458016::{draw_target, drcost, register_draw_summon, track_searches};
use super::c100458017::drop;
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 100458018;

// Copies to draw and to summon, on the field as Level 7 materials for the Rank 7 LIGHT Xyz Monsters in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, DECK),
    (ID, DECK),
    (ID, HAND),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (100458023, EXTRA),
    (100458023, EXTRA),
    (100458022, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    register_draw_summon::<0>(duel, c, ID);
    let e2a = duel.create_effect(c);
    e2a.set_description(duel, aux::stringid(ID, 1));
    e2a.set_category(duel, CATEGORY_DRAW);
    e2a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2a.set_property(duel, EFFECT_FLAG_DELAY);
    e2a.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2a.set_condition(duel, |duel, a| a.e.get_handler(duel).is_summon_location(duel, LOCATION_HAND));
    e2a.set_cost(duel, drcost::<ID, 3, { RESET_PHASE | PHASE_END }>);
    e2a.set_target(duel, draw_target);
    e2a.set_operation(duel, drop::<ID>);
    c.register_effect(duel, e2a);
    let e2b = e2a.clone_effect(duel);
    e2b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2b);
    track_searches::<ID, { RESET_PHASE | PHASE_END }>(duel, c);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e3.set_target(duel, xyztg);
    e3.set_operation(duel, xyzop);
    e3.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e3);
}

fn xyzfilter(c: CardId, duel: &Duel) -> bool {
    c.is_rank(duel, &[7]) && c.is_attribute(duel, ATTRIBUTE_LIGHT) && c.is_xyz_summonable(duel, None, None)
}

fn xyztg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, xyzfilter, LOCATION_EXTRA, CATEGORY_SPECIAL_SUMMON)
}

fn xyzop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, xyzfilter, tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first() {
        duel.xyz_summon(tp, sc, None, None)?;
    }
    Ok(())
}
