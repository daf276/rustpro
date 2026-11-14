// Fairy Tail - Matchgiru
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 19144622;

pub const LISTED_SERIES: &[u16] = &[SET_FAIRY_TAIL];
pub const LISTED_NAMES: &[u32] = &[91957038, CARD_PRINCE_OF_FAIRIES];

// Fairy Tail - Luna on the field to enable the Special Summon, Fairy Tale Tails in the Deck to search, and Elfnote Lucina
// for the opponent to have an Effect Monster.
pub const SUPPORT: &[(u32, u16)] = &[
    (86937530, MZONE),
    (91957038, DECK),
    (91957038, DECK),
    (ID, HAND),
    (ID, GRAVE),
    (ID, MZONE),
    (13597785, MZONE | OPPONENT),
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
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND | LOCATION_GRAVE);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, spcon);
    e1.set_target(duel, stock::special_summon_self_target);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2a = duel.create_effect(c);
    e2a.set_description(duel, aux::stringid(ID, 1));
    e2a.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2a.set_property(duel, EFFECT_FLAG_DELAY);
    e2a.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2a.set_count_limit(duel, 1, (ID, 1));
    e2a.set_target(duel, thtg);
    e2a.set_operation(duel, thop);
    c.register_effect(duel, e2a);
    let e2b = e2a.clone_effect(duel);
    e2b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2b);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_cost(duel, Cost::pay_lp::<500>);
    e3.set_target(duel, namechangetg);
    e3.set_operation(duel, namechangeop);
    c.register_effect(duel, e3);
}

fn spconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_SPELLCASTER) && c.is_base_attack(duel, 1850) && c.is_faceup(duel)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(spconfilter, a.tp, LOCATION_MZONE, LOCATION_MZONE, 1, None)
}

// The effect that stops Special Summons of non-Spellcasters for the rest of the turn, also used by Once Upon a Fairy Tail.
pub fn register_spellcaster_only(duel: &mut Duel, c: CardId, tp: u8, description: u64) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, description);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| !c.is_race(duel, RACE_SPELLCASTER));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    register_spellcaster_only(duel, c, tp, aux::stringid(ID, 3));
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    ((c.is_set_card(duel, &[SET_FAIRY_TAIL]) && c.is_spell_trap(duel)) || c.is_code(duel, 91957038)) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn namechangefilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_EFFECT) && c.is_faceup(duel) && !c.is_code(duel, CARD_PRINCE_OF_FAIRIES)
}

fn namechangetg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(namechangefilter, a.tp, 0, LOCATION_MZONE, 1, None));
    }
    Ok(false)
}

fn namechangeop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_APPLYTO);
    let g = duel.select_matching_card(tp, namechangefilter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    if let Some(sc) = g.get_first() {
        duel.hint_selection(sc);
        become_prince(duel, a.e.get_handler(duel), sc);
    }
    Ok(())
}

// The effect that changes a card's name to "Fairy Prince", also used by the other Fairy Tail cards that do so.
pub fn become_prince(duel: &mut Duel, c: CardId, sc: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EFFECT_CHANGE_CODE);
    e1.set_value(duel, CARD_PRINCE_OF_FAIRIES as i32);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    sc.register_effect(duel, e1);
}
