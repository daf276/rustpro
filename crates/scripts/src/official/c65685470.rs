// Spirit of the Six Samurai
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 65685470;

struct Procedure;

impl UnionProcedure for Procedure {
    const OLD_EQUIP: bool = true;
    const OLD_PROTECT: Option<bool> = Some(false);

    fn f(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_SIX_SAMURAI])
    }
}

pub const LISTED_SERIES: &[u16] = &[SET_SIX_SAMURAI];
// On the field with Six Samurai (Kizan, Kageki) to equip, which then destroy the opponent's Sangan in battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (49721904, MZONE),
    (2511717, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::ResolvesEffect(aux::stringid(ID, 2))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Union::add_procedure::<Procedure>(duel, c);
    for code in [EFFECT_UPDATE_ATTACK, EFFECT_UPDATE_DEFENSE] {
        let e = duel.create_effect(c);
        e.set_type(duel, EFFECT_TYPE_EQUIP);
        e.set_code(duel, code);
        e.set_value(duel, 500);
        e.set_condition(duel, Union::is_union_state);
        c.register_effect(duel, e);
    }
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_category(duel, CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_BATTLE_DESTROYING);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_condition(duel, drcon);
    e1.set_target(duel, drtg);
    e1.set_operation(duel, drop);
    c.register_effect(duel, e1);
}

fn drcon(duel: &Duel, a: &Args) -> bool {
    Union::is_union_state(duel, a) && a.e.get_handler(duel).get_equip_target(duel) == a.eg.and_then(|eg| eg.get_first())
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(a.tp);
    duel.set_target_param(1);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, a.tp, 1);
    Ok(false)
}

fn drop(duel: &mut Duel, a: &Args) -> Script<()> {
    if !a.e.get_handler(duel).is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.draw(p, d, REASON_EFFECT)?;
    Ok(())
}
