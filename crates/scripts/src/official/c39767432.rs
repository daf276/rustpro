// Sorcerer of Sebek
use crate::prelude::*;
use crate::testing::{Check, MZONE};

const ID: u32 = 39767432;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }
}

pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_RECOVER);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EVENT_BATTLE_DAMAGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, lpcon);
    e1.set_target(duel, lptg);
    e1.set_operation(duel, lpop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DAMAGE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EVENT_RECOVER);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, damtg);
    e2.set_operation(duel, damop);
    c.register_effect(duel, e2);
}

fn lpcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    a.eg.and_then(|eg| eg.get_first())
        .is_some_and(|rc| a.ep == 1 - a.tp && rc.is_controler(duel, a.tp) && (rc == c || c.get_linked_group(duel).contains(rc)))
}

fn lptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(a.tp);
    duel.set_target_param(a.ev);
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, a.tp, a.ev as i32);
    Ok(false)
}

fn lpop(duel: &mut Duel, _: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.recover(p, d, REASON_EFFECT)?;
    Ok(())
}

fn damtg(duel: &mut Duel, _: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, PLAYER_ALL, 1000);
    Ok(false)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.damage_full(a.tp, 1000, REASON_EFFECT, true, None)?;
    duel.damage_full(1 - a.tp, 1000, REASON_EFFECT, true, None)?;
    duel.rd_complete()?;
    Ok(())
}
