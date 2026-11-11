// Crystal Clear Wing Over Synchro Dragon
use super::c4891376::valcheck;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 84343351;
const CLEAR_WING_SYNCHRO_DRAGON: u32 = 82044279;

pub const LISTED_NAMES: &[u32] = &[CLEAR_WING_SYNCHRO_DRAGON];
pub const MATERIAL: &[u32] = &[CLEAR_WING_SYNCHRO_DRAGON];

struct Tuners;

impl SynchroProcedure for Tuners {
    const MIN1: usize = 2;
    const MAX1: usize = 99;
    const MIN2: usize = 1;
    const MAX2: usize = 1;

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_summon_code(duel, Some(sc), sumtype, tp, &[CLEAR_WING_SYNCHRO_DRAGON])
    }
}

struct SynchroTuner;

impl SynchroProcedure for SynchroTuner {
    const MIN2: usize = 1;
    const MAX2: usize = 1;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_SYNCHRO, Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        Tuners::f2(c, duel, sc, sumtype, tp)
    }
}

// Synchro Summoned with Accel Synchron (a Level 5 Synchro Tuner) and Clear Wing Synchro Dragon on the field, with the opponent's
// Dark Hole and Raigeki to negate and destroy it, and a Clear Wing in the Extra Deck to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (CLEAR_WING_SYNCHRO_DRAGON, EXTRA),
    (CLEAR_WING_SYNCHRO_DRAGON, MZONE),
    (37675907, MZONE),
    (53129443, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 2)),
    Check::ResolvesEffect(aux::stringid(ID, 3)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let synchro_proc0 = Synchro::add_procedure::<Tuners>(duel, c);
    let synchro_proc1 = Synchro::add_procedure::<SynchroTuner>(duel, c);
    synchro_proc0.set_description(duel, aux::stringid(ID, 0));
    synchro_proc1.set_description(duel, aux::stringid(ID, 1));
    c.add_must_be_synchro_summoned(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY + CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, |duel, a| {
        !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED) && duel.is_chain_negatable(a.ev as u8)
    });
    e1.set_target(duel, negtg);
    e1.set_operation(duel, negop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 3));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_LEAVE_FIELD);
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_MATERIAL_CHECK);
    e3.set_value(duel, Value::MaterialCheck(valcheck));
    c.register_effect(duel, e3);
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let Some(re) = a.re else { return Ok(false) };
    let rc = re.get_handler(duel);
    if chk == 0 {
        return Ok(rc != a.e.get_handler(duel));
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, tp, 0);
    if rc.is_relate_to_effect(duel, re) && rc.is_destructable(duel) {
        duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, tp, 0);
    }
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let Some(re) = a.re else { return Ok(()) };
    let rc = re.get_handler(duel);
    if duel.negate_activation(a.ev as u8) && rc.is_relate_to_effect(duel, re) {
        let atk = if rc.is_monster(duel) { rc.get_text_attack(duel).max(0) } else { 0 };
        let c = e.get_handler(duel);
        if duel.destroy(rc, REASON_EFFECT)? > 0 && atk > 0 && c.is_relate_to_effect(duel, e) && c.is_faceup(duel) {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_UPDATE_ATTACK);
            e1.set_value(duel, atk);
            e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END);
            c.register_effect(duel, e1);
        }
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_position(duel, POS_FACEUP) && c.is_synchro_summoned(duel) && c.is_previous_controler(duel, a.tp) && a.rp == 1 - a.tp
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_CLEAR_WING])
        && duel.get_location_count_from_ex(tp, tp, None, c) > 0
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
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
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
