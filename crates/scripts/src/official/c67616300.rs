// Chicken Game
use crate::prelude::*;
use crate::testing::{Check, HAND};

const ID: u32 = 67616300;

pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EFFECT_CHANGE_DAMAGE);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_target_range(duel, 1, 0);
    e2.set_condition(duel, damcon1);
    e2.set_value(duel, 0);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_NO_EFFECT_DAMAGE);
    c.register_effect(duel, e3);
    let e4 = e2.clone_effect(duel);
    e4.set_target_range(duel, 0, 1);
    e4.set_condition(duel, damcon2);
    c.register_effect(duel, e4);
    let e5 = e4.clone_effect(duel);
    e5.set_code(duel, EFFECT_NO_EFFECT_DAMAGE);
    c.register_effect(duel, e5);
    let e6 = duel.create_effect(c);
    e6.set_description(duel, aux::stringid(ID, 0));
    e6.set_type(duel, EFFECT_TYPE_IGNITION);
    e6.set_property(duel, EFFECT_FLAG_BOTH_SIDE);
    e6.set_range(duel, LOCATION_FZONE);
    e6.set_count_limit(duel, 1, 0);
    e6.set_cost(duel, Cost::pay_lp::<1000>);
    e6.set_target(duel, efftg);
    e6.set_operation(duel, effop);
    c.register_effect(duel, e6);
}

fn damcon1(duel: &Duel, a: &Args) -> bool {
    let tp = a.e.get_handler_player(duel);
    duel.get_lp(tp) < duel.get_lp(1 - tp)
}

fn damcon2(duel: &Duel, a: &Args) -> bool {
    let tp = a.e.get_handler_player(duel);
    duel.get_lp(1 - tp) < duel.get_lp(tp)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(true);
    }
    let b1 = duel.is_player_can_draw(tp, 1);
    let op = duel.select_effect(
        tp,
        &[
            (b1, aux::stringid(ID, 1)),
            (true, aux::stringid(ID, 2)),
            (true, aux::stringid(ID, 3)),
        ],
    )?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    match op {
        Some(1) => {
            e.set_category(duel, CATEGORY_DRAW);
            e.set_property(duel, EFFECT_FLAG_BOTH_SIDE + EFFECT_FLAG_PLAYER_TARGET);
            duel.set_target_player(tp);
            duel.set_target_param(1);
            duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
        }
        Some(2) => {
            e.set_category(duel, CATEGORY_DESTROY);
            e.set_property(duel, EFFECT_FLAG_BOTH_SIDE);
            duel.set_operation_info(0, CATEGORY_DESTROY, Some(e.get_handler(duel).into()), 1, tp, 0);
        }
        Some(3) => {
            e.set_category(duel, CATEGORY_RECOVER);
            e.set_property(duel, EFFECT_FLAG_BOTH_SIDE + EFFECT_FLAG_PLAYER_TARGET);
            duel.set_target_player(1 - tp);
            duel.set_target_param(1000);
            duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, 1 - tp, 1000);
        }
        _ => {}
    }
    duel.set_chain_limit(|_, _, _, _, _| false, 0);
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    match e.get_label(duel) {
        1 => {
            let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
            let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
            duel.draw(p, d, REASON_EFFECT)?;
        }
        2 => {
            let c = e.get_handler(duel);
            if c.is_relate_to_effect(duel, e) {
                duel.destroy(c, REASON_EFFECT)?;
            }
        }
        3 => {
            let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
            let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
            duel.recover(p, d, REASON_EFFECT)?;
        }
        _ => {}
    }
    Ok(())
}
