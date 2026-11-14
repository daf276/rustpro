// DoomZ Command "D.O.O.M.D.U.R.G."
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 68831625;

pub const LISTED_SERIES: &[u16] = &[SET_DOOMZ];

struct Procedure;

impl EquipProcedure for Procedure {}

// In the hand, to equip DoomZ monsters (Amalthe, Elara, Diactorus) on the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (28877382, MZONE),
    (28877382, MZONE),
    (54265980, MZONE),
    (54265980, MZONE),
    (31010081, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesGranted(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Equip::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).get_equip_target(duel).is_some());
    e1.set_target(duel, damtg);
    e1.set_operation(duel, damop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e2.set_value(duel, Value::EffectPlayer(aux::tgoval));
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DESTROY + CATEGORY_ATKCHANGE);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_hint_timing(
        duel,
        TIMING_DAMAGE_STEP,
        TIMING_DAMAGE_STEP | TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E,
    );
    e3.set_condition(duel, aux::stat_change_damage_step_condition);
    e3.set_target(duel, desatktg);
    e3.set_operation(duel, desatkop);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_GRANT);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e4.set_target_card(duel, |duel, e, c| {
        e.get_handler(duel).get_equip_target(duel) == Some(c) && c.is_set_card(duel, &[SET_DOOMZ])
    });
    e4.set_label_object(duel, Some(e2));
    c.register_effect(duel, e4);
    let e5 = e4.clone_effect(duel);
    e5.set_label_object(duel, Some(e3));
    c.register_effect(duel, e5);
    let e6 = e2.clone_effect(duel);
    e6.set_type(duel, EFFECT_TYPE_XMATERIAL);
    e6.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e6.set_range(duel, LOCATION_MZONE);
    c.register_effect(duel, e6);
    let e7 = e3.clone_effect(duel);
    e7.set_type(duel, EFFECT_TYPE_XMATERIAL + EFFECT_TYPE_QUICK_O);
    e7.set_range(duel, LOCATION_MZONE);
    e7.set_condition(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        c.is_attribute(duel, ATTRIBUTE_WIND) && c.is_race(duel, RACE_MACHINE)
    });
    c.register_effect(duel, e7);
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let Some(p) = a.e.get_handler(duel).get_equip_target(duel).map(|ec| ec.get_controler(duel)) else {
        return Ok(false);
    };
    duel.set_target_player(p);
    duel.set_target_param(500);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 1, p, 500);
    Ok(false)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(ec) = a.e.get_handler(duel).get_equip_target(duel) {
        let p = ec.get_controler(duel);
        duel.damage(p, 500, REASON_EFFECT)?;
    }
    Ok(())
}

fn level_or_rank_atk(c: CardId, duel: &Duel) -> i32 {
    (if c.has_level(duel) { c.get_level(duel) } else { c.get_rank(duel) }) as i32 * 100
}

fn desatktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let dg = duel.get_matching_group(CardId::is_faceup, tp, LOCATION_ONFIELD, 0, Some(c.into()));
    if chk == 0 {
        return Ok(!dg.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(dg), 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_ATKCHANGE, None, 1, tp, level_or_rank_atk(c, duel));
    Ok(false)
}

fn desatkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let related = c.is_relate_to_effect(duel, e);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let desg = duel.select_matching_card(tp, CardId::is_faceup, tp, LOCATION_ONFIELD, 0, 1, 1, related.then_some(c.into()))?;
    if desg.is_empty() {
        return Ok(());
    }
    duel.hint_selection(desg);
    if duel.destroy(desg, REASON_EFFECT)? > 0 && c.is_relate_to_effect(duel, e) {
        if c.is_faceup(duel) {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_UPDATE_ATTACK);
            e1.set_value(duel, level_or_rank_atk(c, duel));
            e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END);
            c.register_effect(duel, e1);
        }
        let e2 = duel.create_effect(c);
        e2.set_description(duel, aux::stringid(ID, 2));
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
        e2.set_code(duel, EFFECT_DIRECT_ATTACK);
        e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e2);
        let e3 = duel.create_effect(c);
        e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
        e3.set_code(duel, EVENT_DAMAGE_STEP_END);
        e3.set_condition(duel, |duel, a| a.e.get_handler(duel).is_relate_to_battle(duel));
        e3.set_operation(duel, |duel, a| {
            let c = a.e.get_handler(duel);
            duel.hint(HINT_CARD, 0, c.get_original_code_rule(duel).into());
            duel.destroy(c, REASON_EFFECT)?;
            Ok(())
        });
        e3.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e3);
    }
    Ok(())
}
