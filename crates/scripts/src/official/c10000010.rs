// The Winged Dragon of Ra
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 10000010;
const MILLENNIUM_REVELATION: u32 = 41044418;

// In the hand to be Tribute Summoned with three Prevent Rats.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (549481, MZONE),
    (549481, MZONE),
    (549481, MZONE),
    (549481, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::TributeSummoned(3),
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct SummonProcedure;

impl NormalProcedure for SummonProcedure {
    const NS: bool = true;
    const OPT: bool = false;
    const MIN: i32 = 3;
    const MAX: i32 = 3;
}

struct SetProcedure;

impl NormalProcedure for SetProcedure {
    const NS: bool = false;
    const OPT: bool = false;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Normal::add_summon_procedure::<SummonProcedure>(duel, c);
    Normal::add_set_procedure::<SetProcedure>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e0.set_value(duel, Value::SpSummonCondition(spval));
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_CANNOT_DISABLE_SUMMON);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_operation(duel, |duel, a| {
        let handler = a.e.get_handler(duel).index() as i64;
        duel.set_chain_limit_till_chain_end(|duel, e, _, _, handler| e.get_handler(duel).index() as i64 == handler, handler);
        Ok(())
    });
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_ATKCHANGE);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_SUMMON_SUCCESS);
    e3.set_cost(duel, atkcost);
    e3.set_operation(duel, atkop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_DESTROY);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_cost(duel, Cost::pay_lp::<1000>);
    e4.set_target(duel, destg);
    e4.set_operation(duel, desop);
    c.register_effect(duel, e4);
}

fn spval(duel: &Duel, e: EffectId, _: Option<EffectId>, sp: u8, st: u32, _: u8, _: u8) -> bool {
    st == SUMMON_TYPE_SPECIAL + SUMMON_WITH_MONSTER_REBORN
        && duel.is_player_affected_by_effect(sp, MILLENNIUM_REVELATION).is_some()
        && e.get_handler(duel).is_controler(duel, sp)
}

fn atkcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let lp_cost = duel.get_lp(a.tp) - 100;
    if chk == 0 {
        return Ok(lp_cost > 0 && duel.check_lp_cost(a.tp, lp_cost as u32));
    }
    a.e.set_label(duel, i64::from(lp_cost));
    duel.pay_lp_cost(a.tp, lp_cost as u32)?;
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let c = e.get_handler(duel);
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, e) {
        let label = e.get_label(duel);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_range(duel, LOCATION_MZONE);
        e1.set_value(duel, label as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
        c.register_effect(duel, e2);
    }
    Ok(())
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
