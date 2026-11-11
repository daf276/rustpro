// Linkuriboh
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 41999284;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 1;

    fn f(c: CardId, duel: &Duel, _: CardId, _: u64, _: u8) -> bool {
        c.is_level(duel, &[1])
    }
}

// In the Extra Deck for its Link Summon from Kuriboh, on the field to tribute against the opponent's attacks, and in
// the Graveyard to Special Summon by releasing a Level 1 monster.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (ID, GRAVE),
    (40640057, MZONE),
    (40640057, MZONE),
    (7892180, MZONE | OPPONENT),
    (59053232, MZONE | OPPONENT),
    (59053232, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, atkcon);
    e1.set_cost(duel, Cost::self_tribute);
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_hint_timing(duel, 0, TIMING_BATTLE_START | TIMING_END_PHASE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, spcost);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e2);
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    duel.get_attacker()
        .is_some_and(|at| duel.is_turn_player(1 - a.tp) && at.has_non_zero_attack(duel))
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tc = duel.get_attacker();
    unported!(duel, tc.is_none());
    let Some(tc) = tc else { return Ok(()) };
    if tc.is_relate_to_battle(duel) && tc.is_faceup(duel) {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_value(duel, 0);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
    }
    Ok(())
}

fn cfilter(c: CardId, duel: &Duel, ft: i32) -> bool {
    c.is_level(duel, &[1]) && (ft > 0 || c.get_sequence(duel) < 5)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    let filter = |c, duel: &Duel| cfilter(c, duel, ft);
    if chk == 0 {
        return Ok(ft > -1 && duel.check_release_group_cost(tp, filter, (1, 1), false, None, None));
    }
    let g = duel.select_release_group_cost(tp, filter, (1, 1), false, None, None)?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_can_be_special_summoned(duel, a.e, 0, a.tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}
