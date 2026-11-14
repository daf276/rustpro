// Toon World the Perfect World
use engine::constants::MSG_SELECT_EFFECTYN;

use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 7293697;

pub const LISTED_SERIES: &[u16] = &[SET_TOON];

// Per player, the count of original codes banished this turn, then the codes.
const BANISHED_NAMES: u64 = (ID as u64) << 16;

pub const LISTED_NAMES: &[u32] = &[CARD_TOON_WORLD];

// In the hand to activate, with Toon monsters on the field to banish and in the Deck to add, and Sangan of the opponent to activate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (91842653, MZONE),
    (65458948, MZONE),
    (59383041, DECK),
    (42386471, DECK),
    (55144522, HAND),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Message(MSG_SELECT_EFFECTYN)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_CHANGE_CODE);
    e1.set_range(duel, LOCATION_FZONE);
    e1.set_value(duel, CARD_TOON_WORLD as i32);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_count_limit(duel, 3, ID);
    e2.set_cost(duel, thcost);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EVENT_CHAIN_SOLVING);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_condition(duel, rmcon);
    e3.set_operation(duel, rmop);
    c.register_effect(duel, e3);
    aux::global_check(duel, ID, |duel| aux::add_values_reset(duel, reset_names));
}

fn reset_names(duel: &mut Duel) {
    duel.set_script_state(BANISHED_NAMES, 0);
    duel.set_script_state(BANISHED_NAMES | 1 << 8, 0);
}

fn banished_name(duel: &Duel, tp: u8, code: u32) -> bool {
    let base = BANISHED_NAMES | u64::from(tp) << 8;
    (1..=duel.get_script_state(base)).any(|slot| duel.get_script_state(base | slot as u64) == i64::from(code))
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(!c.has_flag_effect(duel, ID));
    }
    c.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_set_card(duel, &[SET_TOON]) || c.lists_code_with_archetype(duel, &[SET_TOON])) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn rmfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_type(duel, TYPE_TOON) && c.is_faceup(duel) && c.is_able_to_remove(duel) && !banished_name(duel, tp, c.get_original_code_rule(duel))
}

fn rmcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.re.is_some_and(|re| re.is_activated(duel) && re.get_handler(duel) != a.e.get_handler(duel))
        && duel.is_existing_matching_card(|c, duel| rmfilter(c, duel, tp), tp, LOCATION_MZONE, 0, 1, None)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(|c, duel| rmfilter(c, duel, tp), tp, LOCATION_MZONE, 0, None);
    if g.is_empty() || !duel.select_effect_yes_no_full(tp, e.get_handler(duel), aux::stringid(ID, 1))? {
        return Ok(());
    }
    duel.hint(HINT_CARD, 0, ID.into());
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let Some(sc) = g.select(duel, tp, 1, 1, None)?.get_first() else {
        return Ok(());
    };
    duel.hint_selection(sc);
    let temp_banish_eff =
        aux::remove_until_full::<aux::DefaultFieldReturnOp>(duel, sc, 0, REASON_EFFECT, (PHASE_END, RESET_CHAIN), ID, e, tp, 1, 0)?;
    // Lua errors on a nil effect
    unported!(duel, temp_banish_eff.is_none());
    let Some(temp_banish_eff) = temp_banish_eff else {
        return Ok(());
    };
    let e1 = temp_banish_eff.clone_effect(duel);
    e1.set_code(duel, EVENT_CHAIN_SOLVED);
    duel.register_effect(e1, tp);
    temp_banish_eff.reset(duel);
    let base = BANISHED_NAMES | u64::from(tp) << 8;
    let count = duel.get_script_state(base) + 1;
    duel.set_script_state(base, count);
    duel.set_script_state(base | count as u64, i64::from(sc.get_original_code_rule(duel)));
    Ok(())
}
