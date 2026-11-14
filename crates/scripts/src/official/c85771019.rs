// Darklord Asmodeus
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 85771019;

pub const LISTED_NAMES: &[u32] = &[ID + 1, ID + 2];

const DARKLORD_GULGOLET: u32 = 84031359;
const RAIGEKI: u32 = 12580477;

// On the field, with Fairies in the Deck to send and Raigeki for the opponent to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (DARKLORD_GULGOLET, DECK),
    (DARKLORD_GULGOLET, DECK),
    (RAIGEKI, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(ID + 1)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_SINGLE_RANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_range(duel, LOCATION_DECK | LOCATION_GRAVE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_value(duel, Value::SpSummonCondition(|_, _, _, _, _, _, _| false));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_target(duel, tgtg);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_condition(duel, spcon);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_race(duel, RACE_FAIRY) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, tgfilter, LOCATION_DECK, CATEGORY_TOGRAVE)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::send_to_grave(duel, a, tgfilter, LOCATION_DECK)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_reason(duel, REASON_DESTROY) && c.is_previous_controler(duel, a.tp) && c.is_previous_location(duel, LOCATION_ONFIELD)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 2, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, a.tp, 0);
    Ok(false)
}

fn token(duel: &mut Duel, a: &Args, code: u32, code_effect: u32) -> Script<()> {
    let tp = a.tp;
    let Some(token) = duel.create_token(tp, code) else { return Ok(()) };
    if duel.special_summon_step(token, 0, tp, tp, false, false, POS_FACEUP)? {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, code_effect);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_value(duel, 1);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        token.register_effect(duel, e1);
    }
    Ok(())
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some()
        || duel.get_location_count(tp, LOCATION_MZONE) < 2
        || !duel.is_player_can_special_summon_monster(
            tp,
            ID + 1,
            MonsterStats::new(0, TYPES_TOKEN, 1800, 1300, 5, RACE_FAIRY, ATTRIBUTE_DARK),
        )
        || !duel.is_player_can_special_summon_monster(
            tp,
            ID + 2,
            MonsterStats::new(0, TYPES_TOKEN, 1200, 1200, 3, RACE_FAIRY, ATTRIBUTE_DARK),
        )
    {
        return Ok(());
    }
    token(duel, a, ID + 1, EFFECT_INDESTRUCTABLE_EFFECT)?;
    token(duel, a, ID + 2, EFFECT_INDESTRUCTABLE_BATTLE)?;
    duel.special_summon_complete()?;
    Ok(())
}
