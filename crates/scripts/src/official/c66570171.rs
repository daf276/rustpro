// Necroworld Banshee
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 66570171;
const ZOMBIE_WORLD: u32 = 4064256;

pub const LISTED_NAMES: &[u32] = &[ZOMBIE_WORLD];

// On the field, with Zombie World in the hand and Deck to activate, and the opponent's Mystical Space Typhoon to target it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, HAND),
    (ZOMBIE_WORLD, DECK),
    (ZOMBIE_WORLD, DECK),
    (ZOMBIE_WORLD, HAND),
    (5318639, HAND | OPPONENT),
    (5318639, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_FZONE, LOCATION_FZONE);
    e1.set_target_card(duel, |duel, _, c| c.is_code(duel, ZOMBIE_WORLD));
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_range(duel, LOCATION_MZONE | LOCATION_GRAVE);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_count_limit(duel, 1, ID);
    e3.set_cost(duel, Cost::self_banish);
    e3.set_target(duel, actg);
    e3.set_operation(duel, acop);
    c.register_effect(duel, e3);
}

fn filter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_code(duel, ZOMBIE_WORLD)
        && c.get_activate_effect(duel)
            .is_some_and(|te| te.is_activatable(duel, tp, true, true))
}

fn actg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    Ok(chk == 0 && duel.is_existing_matching_card(|c, duel| filter(c, duel, tp), tp, LOCATION_HAND | LOCATION_DECK, 0, 1, None))
}

fn acop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    let g = duel.select_matching_card(tp, |c, duel| filter(c, duel, tp), tp, LOCATION_HAND | LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        duel.activate_field_spell(tc, tp)?;
    }
    Ok(())
}
