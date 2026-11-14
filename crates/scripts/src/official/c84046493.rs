// Ghost Beef
use crate::prelude::*;
use crate::testing::{Check, PZONE};

const ID: u32 = 84046493;

pub const ROLL_DICE: bool = true;
pub const SUPPORT: &[(u32, u16)] = &[(ID, PZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DICE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, sctg);
    e1.set_operation(duel, scop);
    c.register_effect(duel, e1);
}

fn sctg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(a.e.get_handler(duel).get_left_scale(duel) < 10);
    }
    duel.set_operation_info(0, CATEGORY_DICE, None, 0, a.tp, 1);
    Ok(false)
}

fn scop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || c.get_left_scale(duel) >= 10 {
        return Ok(());
    }
    let Some(&dc) = duel.toss_dice(tp, 1)?.first() else {
        return Ok(());
    };
    let sch = (10 - c.get_left_scale(duel) as i32).min(i32::from(dc));
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_LSCALE);
    e1.set_value(duel, sch);
    e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_UPDATE_RSCALE);
    c.register_effect(duel, e2);
    Ok(())
}
