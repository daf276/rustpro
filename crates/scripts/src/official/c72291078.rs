// Mecha Phantom Beast O-Lion
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 72291078;

pub const LISTED_NAMES: &[u32] = &[TOKEN_MECHA_PHANTOM_BEAST];

// In the hand, on the field and in the Graveyard, with Mecha Phantom Beast Warbluran in the hand to Normal Summon and Dark Hole in
// the opponent's hand to send it to the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (ID, GRAVE),
    (31480215, HAND),
    (31480215, HAND),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummonedCode(TOKEN_MECHA_PHANTOM_BEAST),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_condition(duel, indcon);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_cost(duel, Cost::self_banish);
    e3.set_target(duel, target);
    e3.set_operation(duel, operation);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_DELAY);
    e4.set_code(duel, EVENT_TO_GRAVE);
    e4.set_count_limit(duel, 1, ID);
    e4.set_target(duel, sptg);
    e4.set_operation(duel, spop);
    c.register_effect(duel, e4);
}

pub fn token_stats() -> MonsterStats {
    MonsterStats::new(SET_MECHA_PHANTOM_BEAST, TYPES_TOKEN, 0, 0, 3, RACE_MACHINE, ATTRIBUTE_WIND)
}

fn tknfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_TOKEN) || c.is_original_type(duel, TYPE_TOKEN)
}

pub fn indcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(tknfilter, a.e.get_handler_player(duel), LOCATION_ONFIELD, 0, 1, None)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MECHA_PHANTOM_BEAST]) && c.is_summonable(duel, true, None)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, a.tp, LOCATION_HAND, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SUMMON, None, 1, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SUMMON);
    if let Some(tc) = duel.select_matching_card(tp, filter, tp, LOCATION_HAND, 0, 1, 1, None)?.get_first() {
        duel.summon(tp, tc, true, None)?;
    }
    Ok(())
}

pub fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_player_can_special_summon_monster(tp, TOKEN_MECHA_PHANTOM_BEAST, token_stats()));
    }
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, 0);
    Ok(false)
}

pub fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_player_can_special_summon_monster(tp, TOKEN_MECHA_PHANTOM_BEAST, token_stats())
        && let Some(token) = duel.create_token(tp, TOKEN_MECHA_PHANTOM_BEAST)
    {
        duel.special_summon(token, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
