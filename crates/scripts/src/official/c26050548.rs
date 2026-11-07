// Extox Hydra
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 26050548;

struct Materials;

impl FusionMixRep for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const FUN1: FusionMaterial = FusionMaterial::Filter(ffilter);
    const MINC: i32 = 2;
    const MAXC: i32 = 99;
}

// In the Extra Deck with Polymerization, and on the field to attack.
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (24094653, HAND), (ID, MZONE)];
// Its draw needs 1000 or more battle damage from it.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_rep::<Materials>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_code(duel, EFFECT_MATERIAL_CHECK);
    e0.set_value(duel, Value::MaterialCheck(matcheck));
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 0, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).has_flag_effect(duel, ID));
    e1.set_target_card(duel, |duel, e, c| {
        let typ = e.get_handler(duel).get_flag_effect_label(duel, ID).unwrap_or(0);
        c.is_type(duel, typ as u32)
    });
    e1.set_value(duel, Value::Card(|duel, _, c| -c.get_base_attack(duel)));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EVENT_BATTLE_DAMAGE);
    e2.set_condition(duel, |_, a| a.ep == 1 - a.tp && a.ev >= 1000);
    e2.set_target(duel, drtg);
    e2.set_operation(duel, drop);
    c.register_effect(duel, e2);
}

fn ffilter(c: CardId, duel: &Duel, _: CardId, _: u64, tp: u8) -> bool {
    c.is_controler(duel, tp) && c.is_location(duel, LOCATION_MZONE) && c.is_summon_location(duel, LOCATION_EXTRA)
}

fn matcheck(duel: &mut Duel, _: EffectId, c: CardId) {
    let typ = c.get_material(duel).get_bitwise_or(duel, |c, duel| c.get_type(duel).into()) as u32 & (TYPE_EXTRA | TYPE_PENDULUM);
    c.register_flag_effect_full(duel, ID, RESET_EVENT | (RESETS_STANDARD & !RESET_TOFIELD), 0, 1, typ.into(), 0);
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (tp, ct) = (a.tp, a.ev / 1000);
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, ct as usize));
    }
    duel.set_target_player(tp);
    duel.set_target_param(ct);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, ct as i32);
    Ok(false)
}

fn drop(duel: &mut Duel, _: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.draw(p, d, REASON_EFFECT)?;
    Ok(())
}
