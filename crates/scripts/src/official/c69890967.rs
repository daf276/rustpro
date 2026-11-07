// Raviel, Lord of Phantasms
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 69890967;

pub const LISTED_NAMES: &[u32] = &[ID + 1];

// A copy in the hand, Special Summoned by Tributing three Fiends, one on the field to Tribute for its effect.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, MZONE), (26202165, MZONE), (26202165, MZONE), (96345184, MZONE)];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_value(duel, Value::SpSummonCondition(|_, _, _, _, _, _, _| false));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_SPSUMMON_PROC);
    e2.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_code(duel, EVENT_SUMMON_SUCCESS);
    e3.set_condition(duel, |_, a| a.ep != a.tp);
    e3.set_target(duel, tktg);
    e3.set_operation(duel, tkop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_ATKCHANGE);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, 0);
    e4.set_cost(duel, atcost);
    e4.set_operation(duel, atop);
    c.register_effect(duel, e4);
}

fn rfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_race(duel, RACE_FIEND) && (c.is_controler(duel, tp) || c.is_faceup(duel))
}

fn release_group(duel: &Duel, tp: u8) -> CardSet {
    duel.get_release_group(tp).filter(duel, |c, duel| rfilter(c, duel, tp), None)
}

fn rescon(duel: &Duel, sg: CardSet, tp: u8) -> (bool, bool) {
    (duel.get_mzone_count(tp, Some(sg)) >= 1, false)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    aux::select_unselect_group_check(duel, release_group(duel, tp), 3, 3, &|duel, sg, _, _| rescon(duel, sg, tp))
}

fn sptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let rg = release_group(duel, tp);
    let mg = aux::select_unselect_group_full(
        duel,
        rg,
        3,
        3,
        &|duel, sg, _, _| rescon(duel, sg, tp),
        tp,
        HINTMSG_RELEASE,
        None,
        true,
    )?;
    if mg.len() == 3 {
        a.e.set_label_object_group(duel, Some(mg));
        return Ok(true);
    }
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(g) = a.e.get_label_object_group(duel) {
        duel.release(g, REASON_COST)?;
    }
    Ok(())
}

fn tktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 1, a.tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, a.tp, 0);
    Ok(false)
}

fn tkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let stats = MonsterStats::new(0, TYPES_TOKEN, 1000, 1000, 1, RACE_FIEND, ATTRIBUTE_DARK);
    if duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_player_can_special_summon_monster(tp, ID + 1, stats) {
        let Some(token) = duel.create_token(tp, ID + 1) else {
            return Ok(());
        };
        duel.special_summon(token, 0, tp, tp, false, false, POS_FACEUP)?;
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CANNOT_ATTACK_ANNOUNCE);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        token.register_effect_full(duel, e1, true);
    }
    Ok(())
}

fn atkfilter(c: CardId, duel: &Duel) -> bool {
    c.get_text_attack(duel) > 0
}

fn atcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, atkfilter, (1, 1), false, None, Some(c.into())));
    }
    let g = duel.select_release_group_cost(tp, atkfilter, (1, 1), false, None, Some(c.into()))?;
    let atk = g.get_first().map_or(0, |tc| tc.get_text_attack(duel)).max(0);
    a.e.set_label(duel, i64::from(atk));
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn atop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let c = e.get_handler(duel);
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, e) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, e.get_label(duel) as i32);
        e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END);
        c.register_effect(duel, e1);
    }
    Ok(())
}
