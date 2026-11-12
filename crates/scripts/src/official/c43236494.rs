// Fairy Tale Prologue: Journey's Dawn
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 43236494;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the hand, with Decoy, a LIGHT Beast, on the field to draw, and Umi, a Field Spell, in the Deck to place.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (20240828, MZONE), (22702055, DECK), (22702055, DECK)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(0),
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CHANGE_BATTLE_DAMAGE);
    e1.set_range(duel, LOCATION_FZONE);
    e1.set_target_range(duel, 1, 1);
    e1.set_value(duel, HALF_DAMAGE as i32);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, drcon);
    e2.set_target(duel, stock::draw_target::<1>);
    e2.set_operation(duel, stock::draw);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e3.set_cost(duel, Cost::self_to_grave);
    e3.set_target(duel, pltg);
    e3.set_operation(duel, plop);
    c.register_effect(duel, e3);
}

fn drconfilter(c: CardId, duel: &Duel) -> bool {
    ((c.is_attribute(duel, ATTRIBUTE_LIGHT) && c.is_race(duel, RACE_BEAST))
        || (c.is_level(duel, &[7, 8]) && c.is_race(duel, RACE_DRAGON) && c.is_type(duel, TYPE_SYNCHRO)))
        && c.is_faceup(duel)
}

fn drcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(drconfilter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn plfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_field_spell(duel) && !c.is_forbidden(duel) && c.check_unique_on_field(duel, tp) && !c.is_code(duel, ID)
}

fn pltg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    Ok(chk == 0 && duel.is_existing_matching_card(|c, duel| plfilter(c, duel, tp), tp, LOCATION_HAND | LOCATION_DECK, 0, 1, None))
}

fn plop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    let g = duel.select_matching_card(
        tp,
        |c, duel| plfilter(c, duel, tp),
        tp,
        LOCATION_HAND | LOCATION_DECK,
        0,
        1,
        1,
        None,
    )?;
    if let Some(sc) = g.get_first() {
        duel.place_field_spell(sc, tp)?;
    }
    Ok(())
}
