// Cyberse Accelerator
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 89238128;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        !c.is_type_full(duel, TYPE_TOKEN, Some(lc), sumtype, tp)
    }
}

// Cyberse Normal Monsters (Protron, Bitron) to Link Summon with and to attack with.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (24154052, MZONE),
    (24154052, MZONE),
    (36211150, MZONE),
    (36211150, MZONE),
    (32295838, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| {
        duel.is_battle_phase() && aux::stat_change_damage_step_condition(duel, a)
    });
    e1.set_cost(duel, cost);
    e1.set_target(duel, atktg);
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 2));
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, _| duel.is_battle_phase());
    e2.set_cost(duel, cost);
    e2.set_target(duel, extg);
    e2.set_operation(duel, exop);
    c.register_effect(duel, e2);
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.get_attack_announced_count(duel) == 0);
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_OATH);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    c.register_effect(duel, e1);
    Ok(false)
}

fn atkfilter(c: CardId, duel: &Duel, lg: CardSet) -> bool {
    c.is_faceup(duel) && c.is_race(duel, RACE_CYBERSE) && lg.is_contains(c)
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let lg = a.e.get_handler(duel).get_linked_group(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && atkfilter(chkc, duel, lg));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| atkfilter(c, duel, lg), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, |c, duel| atkfilter(c, duel, lg), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, a.e) || tc.is_facedown(duel) {
        return Ok(());
    }
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, 2000);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    tc.register_effect(duel, e1);
    Ok(())
}

fn exfilter(c: CardId, duel: &Duel, lg: CardSet) -> bool {
    c.is_faceup(duel) && c.is_race(duel, RACE_CYBERSE) && c.get_effect_count(duel, EFFECT_EXTRA_ATTACK_MONSTER) == 0 && lg.is_contains(c)
}

fn extg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let lg = a.e.get_handler(duel).get_linked_group(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && exfilter(chkc, duel, lg));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| exfilter(c, duel, lg), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, |c, duel| exfilter(c, duel, lg), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    Ok(false)
}

fn exop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_description(duel, 3202);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE + EFFECT_FLAG_CLIENT_HINT);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_EXTRA_ATTACK_MONSTER);
    e1.set_value(duel, 1);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    tc.register_effect(duel, e1);
    Ok(())
}
