// Number C32: Shark Drake Veiss
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 49221191;
const SHARK_DRAKE: u32 = 65676461;

pub const LISTED_NAMES: &[u32] = &[SHARK_DRAKE];

// In the Extra Deck, Xyz Summoned with four Level 4 WATER monsters.
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (2118022, MZONE), (2483611, MZONE), (4042268, MZONE), (5628232, MZONE)];
// Its effect needs 1000 or less LP, which no SUPPORT can set.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1)))];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_attribute_full(duel, ATTRIBUTE_WATER, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(4);
    const CT: usize = 4;
    const ALTERF: Option<AlterFilter> =
        Some(|c, duel, tp, lc| c.is_summon_code(duel, Some(lc), u64::from(SUMMON_TYPE_XYZ), tp, &[SHARK_DRAKE]) && c.is_faceup(duel));
    const DESC: u64 = aux::stringid(ID, 0);
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DEFCHANGE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_hint_timing(
        duel,
        TIMING_DAMAGE_STEP,
        TIMING_DAMAGE_STEP | TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E,
    );
    e1.set_condition(duel, |duel, a| {
        duel.get_lp(a.tp) <= 1000 && aux::stat_change_damage_step_condition(duel, a)
    });
    e1.set_cost(duel, |duel, a, chk| {
        Cost::and(duel, a, chk, &[atkdefcost, Cost::detach_from_self::<1, 1>])
    });
    e1.set_target(duel, atkdeftg);
    e1.set_operation(duel, atkdefop);
    c.register_effect(duel, e1);
}

fn nonzero(c: CardId, duel: &Duel) -> bool {
    c.has_non_zero_attack(duel) || c.has_non_zero_defense(duel)
}

fn atkdefcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel)
        && c.is_able_to_remove_as_cost(duel)
        && aux::sp_elim_filter(c, duel, false, false)
        && duel.is_existing_target(nonzero, 0, LOCATION_MZONE, LOCATION_MZONE, 1, Some(c.into()))
}

fn atkdefcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(atkdefcostfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, atkdefcostfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn atkdeftg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && nonzero(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(nonzero, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATKDEF);
    duel.select_target(tp, nonzero, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    Ok(false)
}

fn atkdefop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel) {
        let ct = if duel.is_turn_player(1 - tp) { 2 } else { 1 };
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_value(duel, 0);
        e1.set_reset_full(duel, RESETS_STANDARD_PHASE_END, ct);
        tc.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_SET_DEFENSE_FINAL);
        tc.register_effect(duel, e2);
    }
    Ok(())
}
