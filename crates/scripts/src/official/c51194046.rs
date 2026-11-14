// Qliphort Monolith
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, PZONE};

const ID: u32 = 51194046;
const TRIBUTE_COUNT: u64 = (ID as u64) << 16;

// In the Pendulum Zone, with Qliphort Scouts (Level 5 Qli) on the field to Tribute for Qliphort Monolith and Scouts in the hand.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (ID, HAND),
    (ID, HAND),
    (65518099, HAND),
    (65518099, MZONE),
    (65518099, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e2.set_property(
        duel,
        EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CANNOT_NEGATE,
    );
    e2.set_range(duel, LOCATION_PZONE);
    e2.set_target_range(duel, 1, 0);
    e2.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| !c.is_set_card(duel, &[SET_QLI]));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_DRAW);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_PHASE + PHASE_END);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3.set_range(duel, LOCATION_PZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, drcon);
    e3.set_target(duel, drtg);
    e3.set_operation(duel, drop);
    c.register_effect(duel, e3);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_SUMMON_SUCCESS);
        ge1.set_operation(duel, checkop);
        duel.register_effect(ge1, 0);
        let ge2 = ge1.clone_effect(duel);
        ge2.set_code(duel, EVENT_MSET);
        duel.register_effect(ge2, 0);
        let ge3 = duel.create_effect(c);
        ge3.set_type(duel, EFFECT_TYPE_FIELD);
        ge3.set_code(duel, EFFECT_MATERIAL_CHECK);
        ge3.set_target_range(duel, LOCATION_HAND, LOCATION_HAND);
        ge3.set_value(duel, Value::MaterialCheck(valcheck));
        duel.register_effect(ge3, 0);
        ge1.set_label_object(duel, Some(ge3));
        ge2.set_label_object(duel, Some(ge3));
        aux::add_values_reset(duel, |duel| {
            duel.set_script_state(TRIBUTE_COUNT, 0);
            duel.set_script_state(TRIBUTE_COUNT | 1, 0);
        });
    });
}

fn checkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = a.eg.and_then(|eg| eg.get_first()) else {
        return Ok(());
    };
    if tc.is_tribute_summoned(duel)
        && let Some(ge3) = a.e.get_label_object(duel)
    {
        let key = TRIBUTE_COUNT | u64::from(tc.get_summon_player(duel));
        duel.set_script_state(key, duel.get_script_state(key) + ge3.get_label(duel));
    }
    Ok(())
}

fn valcheck(duel: &mut Duel, e: EffectId, c: CardId) {
    let ct = c
        .get_material(duel)
        .filter_count(duel, |c, duel| c.is_set_card(duel, &[SET_QLI]), None);
    e.set_label(duel, ct as i64);
}

fn drcon(duel: &Duel, a: &Args) -> bool {
    duel.get_script_state(TRIBUTE_COUNT | u64::from(a.tp)) > 0
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let ct = duel.get_script_state(TRIBUTE_COUNT | u64::from(tp));
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, ct as usize));
    }
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, ct as i32);
    Ok(false)
}

fn drop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !a.e.get_handler(duel).is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    duel.draw(tp, duel.get_script_state(TRIBUTE_COUNT | u64::from(tp)) as u32, REASON_EFFECT)?;
    Ok(())
}
