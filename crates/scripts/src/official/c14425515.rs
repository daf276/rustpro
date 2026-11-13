// Infernity Doom Slinger
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 14425515;

// A Fiend (White Duston) in the hand to Special Summon; the Graveyard effect needs an empty hand, which no SUPPORT can arrange.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, GRAVE), (3557275, HAND), (3557275, HAND)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DAMAGE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| duel.get_field_group_count(a.tp, LOCATION_HAND, 0) == 0);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, |duel, a, chk, _| {
        if chk == 0 {
            return Ok(true);
        }
        duel.set_possible_operation_info(0, CATEGORY_DAMAGE, None, 0, a.tp, 4000);
        Ok(false)
    });
    e2.set_operation(duel, effop);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_FIEND) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, e, tp), LOCATION_HAND)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, e, tp), LOCATION_HAND, POS_FACEUP)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let top_card = duel.get_decktop_group(tp, 1).get_first();
    let op = duel.select_effect(1 - tp, &[(top_card.is_some(), aux::stringid(ID, 2)), (true, aux::stringid(ID, 3))])?;
    if op == Some(1) {
        duel.confirm_decktop(tp, 1)?;
        if top_card.is_some_and(|top_card| top_card.is_monster(duel)) {
            let e1 = duel.create_effect(c);
            e1.set_description(duel, aux::stringid(ID, 4));
            e1.set_type(duel, EFFECT_TYPE_FIELD);
            e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
            e1.set_code(duel, EFFECT_REFLECT_DAMAGE);
            e1.set_target_range(duel, 1, 0);
            e1.set_value(duel, Value::ReflectDamage(|_, _, _, _, r, _, _| r & REASON_EFFECT == REASON_EFFECT));
            e1.set_reset(duel, RESET_PHASE | PHASE_END);
            duel.register_effect(e1, tp);
        } else {
            duel.damage(tp, 4000, REASON_EFFECT)?;
        }
    } else if op == Some(2) {
        let e2a = duel.create_effect(c);
        e2a.set_description(duel, aux::stringid(ID, 5));
        e2a.set_type(duel, EFFECT_TYPE_FIELD);
        e2a.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
        e2a.set_code(duel, EFFECT_CHANGE_DAMAGE);
        e2a.set_target_range(duel, 1, 0);
        e2a.set_value(duel, Value::ChangeDamage(damval));
        e2a.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e2a, tp);
        let e2b = e2a.clone_effect(duel);
        e2b.set_code(duel, EFFECT_NO_EFFECT_DAMAGE);
        duel.register_effect(e2b, tp);
    }
    Ok(())
}

fn damval(_: &mut Duel, _: EffectId, _: Option<EffectId>, val: u32, r: u32, _: u8, _: Option<CardId>) -> u32 {
    if r & REASON_EFFECT > 0 { 0 } else { val }
}
