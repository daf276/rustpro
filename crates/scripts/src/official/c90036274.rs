// Clear Wing Fast Dragon
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE, OPPONENT, PZONE};

const ID: u32 = 90036274;

pub const LISTED_SERIES: &[u16] = &[SET_SPEEDROID];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, |c, duel| c.is_attribute(duel, ATTRIBUTE_WIND))
    }
}

// A copy in the Pendulum Zone and in the Extra Deck, Synchro Summoned with Speedroid Darumaknocker (a Level 4 Tuner) and Speedroid Taketomborg (a Level
// 3 WIND), with Speedroid Tri-Eyed Dice (a Level 3 Tuner) and Noble Knight Artorigus (Level 4) to send, and Dark Hole in the opponent's hand.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (85482105, MZONE),
    (53932291, MZONE),
    (27660735, MZONE),
    (92125819, MZONE),
    (27660735, DECK),
    (53129443, DECK | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    Pendulum::add_procedure_full(duel, c, false, None);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, spcost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DISABLE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_hint_timing(duel, TIMING_DAMAGE_STEP, TIMING_DAMAGE_STEP | TIMINGS_CHECK_MONSTER);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, aux::stat_change_damage_step_condition);
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_condition(duel, pencon);
    e3.set_target(duel, pentg);
    e3.set_operation(duel, penop);
    c.register_effect(duel, e3);
}

fn filter(c: CardId, duel: &Duel, filter_func: fn(CardId, &Duel) -> bool) -> bool {
    c.is_faceup(duel) && c.is_level_above(duel, 0) && c.is_able_to_grave_as_cost(duel) && filter_func(c, duel)
}

fn cfilter1(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SPEEDROID]) && c.is_type(duel, TYPE_TUNER)
}

fn cfilter2(c: CardId, duel: &Duel) -> bool {
    !c.is_type(duel, TYPE_TUNER) || c.is_has_effect(duel, EFFECT_NONTUNER).is_some()
}

fn chk(c: CardId, duel: &Duel, sg: CardSet) -> bool {
    cfilter1(c, duel) && sg.is_exists(duel, cfilter2, 1, Some(c.into()))
}

fn spcost(duel: &mut Duel, a: &Args, chk0: i32) -> Script<bool> {
    let tp = a.tp;
    let g1 = duel.get_matching_group(|c, duel| filter(c, duel, cfilter1), tp, LOCATION_MZONE, 0, None);
    let g2 = duel.get_matching_group(|c, duel| filter(c, duel, cfilter2), tp, LOCATION_MZONE, 0, None);
    let rescon = |duel: &Duel, sg: CardSet, mg: CardSet, c: Option<CardId>| {
        let res = aux::chkf_mmz(tp, 1)(duel, sg, mg, c).0
            && sg.is_exists(duel, |c, duel| chk(c, duel, sg), 1, None)
            && sg.check_with_sum_equal(duel, CardId::get_level, 7, 2, 2);
        (res, false)
    };
    let g = g1.union(g2);
    if chk0 == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > -2
            && !g1.is_empty()
            && !g2.is_empty()
            && aux::select_unselect_group_check(duel, g, 2, 2, &rescon));
    }
    let sg = aux::select_unselect_group(duel, g, 2, 2, &rescon, tp, HINTMSG_TOGRAVE)?;
    duel.send_to_grave(sg, REASON_COST)?;
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn disfilter(c: CardId, duel: &Duel) -> bool {
    c.is_summon_location(duel, LOCATION_EXTRA) && !(c.get_attack(duel) == 0 && c.is_disabled(duel))
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && disfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(disfilter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, disfilter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let c = e.get_handler(duel);
    let tc = duel.get_first_target();
    unported!(duel, tc.is_none());
    let Some(tc) = tc else { return Ok(()) };
    if tc.is_faceup(duel) && tc.is_relate_to_effect(duel, e) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_value(duel, 0);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
        duel.negate_related_chain(tc, RESET_TURN_SET);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_code(duel, EFFECT_DISABLE);
        e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e2);
        let e3 = duel.create_effect(c);
        e3.set_type(duel, EFFECT_TYPE_SINGLE);
        e3.set_code(duel, EFFECT_DISABLE_EFFECT);
        e3.set_value(duel, RESET_TURN_SET as i32);
        e3.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e3);
    }
    Ok(())
}

fn pencon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    a.r & (REASON_EFFECT + REASON_BATTLE) != 0 && c.is_previous_location(duel, LOCATION_MZONE) && c.is_faceup(duel)
}

fn pentg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.check_pendulum_zones(a.tp));
    }
    Ok(false)
}

fn penop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if !duel.check_pendulum_zones(tp) {
        return Ok(());
    }
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) {
        duel.move_to_field(c, tp, tp, LOCATION_PZONE, POS_FACEUP, true)?;
    }
    Ok(())
}
