// Core of Chaos
use crate::prelude::*;
use crate::testing::{Check, HAND, REMOVED};

const ID: u32 = 3806388;

pub const LISTED_NAMES: &[u32] = &[77312273];

// Kewl Tune Clip (DARK) in the hand to banish as the cost, and Shell of Chaos banished to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (43904702, HAND), (43904702, HAND), (77312273, REMOVED)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    initial_effect_for::<ID, ATTRIBUTE_DARK, 77312273>(duel, c);
}

// Shell of Chaos has the same script with LIGHT and the other name.
pub fn initial_effect_for<const CODE: u32, const ATTRIBUTE: u32, const OTHER: u32>(duel: &mut Duel, c: CardId) {
    c.enable_unsummonable(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_value(
        duel,
        Value::SpSummonCondition(|duel, _, se, _, _, _, _| se.is_some_and(|se| se.is_has_type(duel, EFFECT_TYPE_ACTIONS))),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_faceup(duel));
    e2.set_value(duel, LOCATION_REMOVED as i32);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(CODE, 0));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_HAND);
    e3.set_count_limit(duel, 1, CODE);
    e3.set_cost(duel, hspcost::<ATTRIBUTE>);
    e3.set_target(duel, stock::special_summon_self_target_player_0);
    e3.set_operation(duel, hspop::<CODE>);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(CODE, 1));
    e4.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e4.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e4.set_count_limit(duel, 1, (CODE, 1));
    e4.set_target(duel, sptg::<OTHER>);
    e4.set_operation(duel, stock::special_summon_first_target::<POS_FACEUP>);
    c.register_effect(duel, e4);
}

fn hspcostfilter<const ATTRIBUTE: u32>(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE)
        && c.is_able_to_remove_as_cost(duel)
        && (c.is_location(duel, LOCATION_HAND) || aux::sp_elim_filter(c, duel, true, false))
}

fn hspcost<const ATTRIBUTE: u32>(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::banish_cost(
        duel,
        a,
        chk,
        hspcostfilter::<ATTRIBUTE>,
        LOCATION_HAND | LOCATION_MZONE | LOCATION_GRAVE,
    )
}

fn hspop<const CODE: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(CODE, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, splimit);
    duel.register_effect(e1, tp);
    aux::add_temp_lizard_check_full(duel, c, tp, lizfilter);
    Ok(())
}

pub fn splimit(duel: &Duel, _: EffectId, c: CardId, _: u8, _: u32, _: u8, _: u8, _: Option<EffectId>) -> bool {
    c.is_location(duel, LOCATION_EXTRA) && !(c.is_type(duel, TYPE_SYNCHRO) && c.is_attribute(duel, ATTRIBUTE_LIGHT | ATTRIBUTE_DARK))
}

pub fn lizfilter(duel: &Duel, _: EffectId, c: CardId) -> bool {
    !c.is_original_type(duel, TYPE_SYNCHRO) || !c.is_attribute(duel, ATTRIBUTE_LIGHT | ATTRIBUTE_DARK)
}

fn spfilter<const OTHER: u32>(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_faceup(duel) && c.is_code(duel, OTHER) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg<const OTHER: u32>(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let filter = |c, duel: &Duel| spfilter::<OTHER>(c, duel, e, tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_REMOVED) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_target(filter, tp, LOCATION_REMOVED, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, filter, tp, LOCATION_REMOVED, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}
