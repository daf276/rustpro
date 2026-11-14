// Ryu-Ge Realm - Wyrm Winds
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 55154344;

pub const LISTED_SERIES: &[u16] = &[SET_RYU_GE];
pub const LISTED_NAMES: &[u32] = &[ID];
// Copies in the hand to activate, with Genro Ryu-Ge Hakva (Level 10 Wyrm) on the field to be granted the effect.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (56499179, MZONE), (56499179, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesGranted(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    activation::<ID>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_SET_AVAILABLE + EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_code(duel, EFFECT_TO_GRAVE_REDIRECT);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.e.get_handler_player(duel)));
    e1.set_target_card(duel, |duel, e, c| duel.is_player_can_remove(e.get_handler_player(duel), Some(c)));
    e1.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1.set_value(duel, LOCATION_REMOVED as i32);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_ATKCHANGE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e2.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e2.set_cost(duel, atkcost);
    e2.set_target(duel, atktg);
    e2.set_operation(duel, atkop);
    grant(duel, c, e2, efftg::<RACE_WYRM>);
}

// The Realms' unique-on-field rule and activation.
pub fn activation<const CODE: u32>(duel: &mut Duel, c: CardId) {
    c.set_unique_on_field(duel, 1, 0, |c, duel| c.is_code(duel, CODE), LOCATION_ONFIELD);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
}

// The Realms' grant of `e2` to the monsters `efftg` takes, which also become Effect Monsters.
pub fn grant(duel: &mut Duel, c: CardId, e2: EffectId, efftg: fn(&Duel, EffectId, CardId) -> bool) {
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_GRANT);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_target_range(duel, LOCATION_MZONE, 0);
    e3.set_target_card(duel, efftg);
    e3.set_label_object(duel, Some(e2));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD);
    e4.set_code(duel, EFFECT_ADD_TYPE);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_target_range(duel, LOCATION_MZONE, 0);
    e4.set_target_card(duel, efftg);
    e4.set_value(duel, TYPE_EFFECT as i32);
    c.register_effect(duel, e4);
}

pub fn efftg<const RACE: u64>(duel: &Duel, _: EffectId, c: CardId) -> bool {
    (c.is_set_card(duel, &[SET_RYU_GE]) && c.is_type(duel, TYPE_PENDULUM)) || (c.is_level_above(duel, 10) && c.is_original_race(duel, RACE))
}

// The Ryu-Ge cost of returning a face-up card `filter` takes from the field to the bottom of the Deck.
pub fn to_deck_cost(duel: &mut Duel, a: &Args, chk: i32, filter: impl Fn(CardId, &Duel) -> bool) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_ONFIELD, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_ONFIELD, 0, 1, 1, None)?;
    duel.hint_selection(g);
    duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKBOTTOM, REASON_COST)?;
    Ok(false)
}

pub fn is_realm_cost(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RYU_GE]) && c.is_continuous_spell(duel) && c.is_faceup(duel) && c.is_able_to_deck_as_cost(duel)
}

fn atkcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    to_deck_cost(duel, a, chk, |c, duel| {
        is_realm_cost(c, duel) && duel.is_existing_target(CardId::has_non_zero_attack, 0, LOCATION_MZONE, LOCATION_MZONE, 1, Some(c.into()))
    })
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.has_non_zero_attack(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::has_non_zero_attack, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATKDEF);
    let g = duel.select_target(tp, CardId::has_non_zero_attack, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    let atk = g.and_then(|g| g.get_first()).map_or(0, |tc| tc.get_attack(duel));
    duel.set_operation_info(0, CATEGORY_ATKCHANGE, g, 1, tp, -atk);
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    if let Some(tc) = duel.get_first_target()
        && tc.is_faceup(duel)
        && tc.is_relate_to_effect(duel, e)
    {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_value(duel, 0);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
