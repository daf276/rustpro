// Destiny HERO - Destroyer Phoenix Enforcer
use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE};

const ID: u32 = 60461804;
pub const MATERIAL_SETCODE: &[u16] = &[SET_HERO, SET_DESTINY_HERO];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = false;
    const INSF: bool = false;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(matfilter),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_DESTINY_HERO], Some(fc), sumtype, tp)),
    ];
}

// A Destiny HERO monster in the GY to Special Summon and one on the field to destroy.
pub const SUPPORT: &[(u32, u16)] = &[(9411399, GRAVE), (9411399, MZONE)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 0, LOCATION_MZONE);
    e1.set_value(duel, Value::Card(atkval));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E | TIMING_MAIN_END);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, |_, a| a.r & (REASON_EFFECT + REASON_BATTLE) != 0);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn matfilter(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_set_card_full(duel, &[SET_HERO], Some(fc), sumtype, tp) && c.is_level_above(duel, 6)
}

fn atkval(duel: &Duel, e: EffectId, _: CardId) -> i32 {
    let tp = e.get_handler(duel).get_controler(duel);
    duel.get_matching_group_count(|c, duel| c.is_set_card(duel, &[SET_HERO]), tp, LOCATION_GRAVE, 0, None) as i32 * -200
}

fn rescon(tp: u8) -> impl Fn(&Duel, CardSet, CardSet, Option<CardId>) -> (bool, bool) {
    move |duel, sg, _, _| (sg.is_exists(duel, |c, duel| c.is_controler(duel, tp), 1, None), false)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let rg = duel.get_matching_group(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    if chk == 0 {
        return Ok(aux::select_unselect_group_check(duel, rg, 2, 2, &rescon(tp)));
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(rg), 2, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let rg = duel.get_matching_group(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    let g = aux::select_unselect_group(duel, rg, 2, 2, &rescon(tp), tp, HINTMSG_DESTROY)?;
    if g.len() == 2 {
        duel.hint_selection(g);
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, a.tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e1.set_count_limit(duel, 1, 0);
    e1.set_label(duel, i64::from(duel.get_turn_count()));
    e1.set_condition(duel, |duel, a| i64::from(duel.get_turn_count()) != a.e.get_label(duel));
    e1.set_operation(duel, spop1);
    let phase_end = RESET_PHASE | PHASE_END;
    if duel.is_turn_player(tp) {
        e1.set_reset_full(duel, phase_end | RESET_OPPO_TURN, 1);
    } else {
        e1.set_reset_full(duel, phase_end | RESET_SELF_TURN, 1);
    }
    duel.register_effect(e1, tp);
    Ok(())
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_DESTINY_HERO]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn spop1(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(|c, duel| filter(c, duel, e, tp)),
        tp,
        LOCATION_GRAVE,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
