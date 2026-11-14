// Dream Shark
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 6180710;

pub const LISTED_SERIES: &[u16] = &[SET_HEROIC];

// In hand with only WATER monsters (Great White) on the field, and in the Graveyard with burn effects (Hinotama) of the opponent to answer.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (ID, GRAVE),
    (13429800, MZONE),
    (46130346, HAND | OPPONENT),
    (46130346, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_COUNT);
    e1.set_count_limit(duel, 1, 0);
    e1.set_value(duel, Value::Reason(|_, _, _, r, _| r & REASON_BATTLE != 0));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, (ID, 0));
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e2);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e4.set_type(duel, EFFECT_TYPE_QUICK_O);
    e4.set_code(duel, EVENT_CHAINING);
    e4.set_range(duel, LOCATION_GRAVE);
    e4.set_count_limit(duel, 1, (ID, 1));
    e4.set_condition(duel, aux::damcon1);
    e4.set_target(duel, damtg);
    e4.set_operation(duel, damop);
    c.register_effect(duel, e4);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_facedown(duel) || !c.is_attribute(duel, ATTRIBUTE_WATER)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    !duel.is_existing_matching_card(cfilter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn special_summon_target(duel: &mut Duel, a: &Args, chk: i32, location: u32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, location as i32);
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    special_summon_target(duel, a, chk, LOCATION_HAND)
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    special_summon_target(duel, a, chk, LOCATION_GRAVE)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && duel.special_summon_step(c, 0, tp, tp, false, false, POS_FACEUP)? {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_UPDATE_DEFENSE);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        e1.set_value(duel, -1000);
        c.register_effect_full(duel, e1, true);
        let e2 = duel.create_effect(c);
        e2.set_description(duel, 3300);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e2.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
        e2.set_value(duel, LOCATION_REMOVED as i32);
        e2.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
        c.register_effect_full(duel, e2, true);
        let cid = duel.get_chain_info::<u32>(a.ev as u8, CHAININFO_CHAIN_ID);
        let e3 = duel.create_effect(c);
        e3.set_type(duel, EFFECT_TYPE_FIELD);
        e3.set_code(duel, EFFECT_CHANGE_DAMAGE);
        e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
        e3.set_target_range(duel, 1, 0);
        e3.set_label(duel, i64::from(cid));
        e3.set_value(duel, Value::ChangeDamage(damval));
        e3.set_reset(duel, RESET_CHAIN);
        duel.register_effect(e3, tp);
    }
    duel.special_summon_complete()?;
    Ok(())
}

fn damval(duel: &mut Duel, e: EffectId, _: Option<EffectId>, val: u32, r: u32, _: u8, _: Option<CardId>) -> u32 {
    let cc = duel.get_current_chain();
    if cc == 0 || r & REASON_EFFECT == 0 {
        return val;
    }
    let cid = duel.get_chain_info::<u32>(0, CHAININFO_CHAIN_ID);
    if i64::from(cid) != e.get_label(duel) { val } else { 0 }
}
