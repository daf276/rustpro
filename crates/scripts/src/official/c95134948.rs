// Number 99: Utopia Dragonar
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 95134948;
pub const XYZ_NUMBER: u32 = 99;

// In the Extra Deck, Xyz Summoned with three Level 12 Blue-Eyes Ultimate Dragons, with Number 39: Utopia in the Extra Deck to Special Summon
// and the opponent's monsters to attack with.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (23995346, MZONE),
    (23995346, MZONE),
    (23995346, MZONE),
    (84013237, EXTRA),
    (84013237, EXTRA),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(12);
    const CT: usize = 3;
    const MAXCT: usize = Xyz::INFINITE_MATS;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_cost(duel, |duel, a, chk| {
        Cost::and(duel, a, chk, &[Cost::detach_from_self::<2, 2>, spcost])
    });
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_ATKCHANGE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        duel.get_attacker().is_some_and(|ac| ac.is_controler(duel, 1 - a.tp))
    });
    e2.set_target(duel, atktg);
    e2.set_operation(duel, atkop);
    c.register_effect(duel, e2);
    duel.add_custom_activity_counter(
        ID,
        ACTIVITY_SPSUMMON,
        CounterFilter::Card(|duel, c| !c.is_summon_location(duel, LOCATION_EXTRA) || c.is_type(duel, TYPE_XYZ)),
    );
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_ATTACK_ANNOUNCE);
        ge1.set_operation(duel, checkop);
        duel.register_effect(ge1, 0);
    });
}

fn checkop(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.get_attack_target().is_none() {
        duel.register_flag_effect(a.ep, ID, RESET_PHASE | PHASE_END, 0, 1);
        if let Some(ac) = a.eg.unwrap_or_default().get_first() {
            ac.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
        }
    }
    Ok(())
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_custom_activity_count(ID, tp, ACTIVITY_SPSUMMON) == 0
            && (duel.get_flag_effect(tp, ID) == 0 || duel.get_flag_effect(tp, ID) == c.get_flag_effect(duel, ID)));
    }
    let fid = i64::from(c.get_field_id(duel));
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_OATH + EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_code(duel, EFFECT_CANNOT_DIRECT_ATTACK);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, e, c| e.get_label(duel) != i64::from(c.get_field_id(duel)));
    e1.set_label(duel, fid);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 2));
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH + EFFECT_FLAG_CLIENT_HINT);
    e2.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e2.set_target_range(duel, 1, 0);
    e2.set_target_card(duel, |duel, _, c| c.is_location(duel, LOCATION_EXTRA) && !c.is_type(duel, TYPE_XYZ));
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e2, tp);
    aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| !c.is_original_type(duel, TYPE_XYZ));
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_NUMBER])
        && c.xyz_number(duel).is_some_and(|no| (1..=100).contains(&no))
        && duel.get_location_count_from_ex(tp, tp, None, c) > 0
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_XYZ, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(
        duel,
        a,
        chk,
        |c, duel| spfilter(c, duel, a.e, a.tp),
        LOCATION_EXTRA,
        CATEGORY_SPECIAL_SUMMON,
    )
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let Some(sc) = duel
        .select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_EXTRA, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    sc.set_material(duel, None);
    if duel.special_summon(sc, SUMMON_TYPE_XYZ, tp, tp, false, false, POS_FACEUP)? > 0 {
        sc.complete_procedure(duel);
    }
    Ok(())
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let Some(bc) = duel.get_attacker() else { return Ok(false) };
    if chk == 0 {
        return Ok(bc.is_on_field(duel) && bc.has_non_zero_attack(duel));
    }
    a.e.set_label_object_group(duel, Some(bc.into()));
    bc.create_effect_relation(duel, a.e);
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(bc) = e.get_label_object_group(duel).and_then(|g| g.get_first()) else {
        return Ok(());
    };
    if bc.is_relate_to_effect(duel, e) && bc.is_faceup(duel) && bc.is_controler(duel, 1 - tp) {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_value(duel, 0);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        bc.register_effect(duel, e1);
    }
    Ok(())
}
