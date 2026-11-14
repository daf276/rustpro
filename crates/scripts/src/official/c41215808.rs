// Floowandereeze and the Dreaming Town
use crate::prelude::*;
use crate::testing::{DECK, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 41215808;

// Set copies and Robina in hand and Deck to Normal Summon, a copy in the Graveyard with Level 8 Winged Beasts and tributes to Tribute
// Summon them, and face-up monsters of the opponent to turn face-down.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (ID, GRAVE),
    (18940725, HAND),
    (18940725, DECK),
    (61441708, HAND),
    (61441708, HAND),
    (76812113, MZONE),
    (76812113, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_POSITION + CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, poscon);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, postg);
    e2.set_operation(duel, posop);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_level_below(duel, 4) && c.is_race(duel, RACE_WINGEDBEAST) && c.is_summonable(duel, true, None)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, a.tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SUMMON, None, 1, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SUMMON);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        duel.summon(tp, tc, true, None)?;
    }
    Ok(())
}

fn poscon(duel: &Duel, a: &Args) -> bool {
    let Some(ec) = a.eg.and_then(|eg| eg.get_first()) else {
        return false;
    };
    ec.is_faceup(duel) && ec.is_summon_player(duel, a.tp) && ec.is_tribute_summoned(duel) && ec.is_level_above(duel, 7)
}

fn posfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_can_turn_set(duel)
}

fn postg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(posfilter, tp, 0, LOCATION_MZONE, None);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_POSITION, Some(g), g.len() as u8, tp, POS_FACEDOWN_DEFENSE as i32);
    Ok(false)
}

fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(posfilter, a.tp, 0, LOCATION_MZONE, None);
    if !g.is_empty() {
        duel.change_position(g, POS_FACEDOWN_DEFENSE)?;
    }
    Ok(())
}
