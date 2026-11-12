// Number C9: Chaos Dyson Sphere
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 32559361;
pub const XYZ_NUMBER: u32 = 9;
const DYSON_SPHERE: u32 = 1992816;

pub const LISTED_NAMES: &[u32] = &[DYSON_SPHERE];

// On the field to battle and attach, and in the Extra Deck (Xyz Summoned with three Level 10 monsters) to damage with the
// attached materials, with the opponent's monsters to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (10000, MZONE),
    (6007213, MZONE),
    (15771991, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (14558127, MZONE | OPPONENT),
    (14558127, MZONE | OPPONENT),
];
// The 800 damage effect needs "Number 9: Dyson Sphere" attached, which no scenario can do.
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 2))),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(10);
    const CT: usize = 3;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BATTLE_START);
    e1.set_target(duel, attachtg);
    e1.set_operation(duel, attachop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DAMAGE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_target(duel, dam300tg);
    e2.set_operation(duel, dam300op);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_DAMAGE);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |duel, a| {
        let overlay = a.e.get_handler(duel).get_overlay_group(duel);
        overlay.is_exists(duel, |c, duel| c.is_code(duel, DYSON_SPHERE), 1, None)
    });
    e3.set_cost(duel, dam800cost);
    e3.set_target(duel, dam800tg);
    e3.set_operation(duel, dam800op);
    c.register_effect(duel, e3);
}

fn attachtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        let c = a.e.get_handler(duel);
        return Ok(c
            .get_battle_target(duel)
            .is_some_and(|bc| bc.is_controler(duel, 1 - a.tp) && bc.is_can_be_xyz_material(duel, Some(c), a.tp, REASON_EFFECT)));
    }
    Ok(false)
}

fn attachop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    let bc = c.get_battle_target(duel);
    unported!(duel, bc.is_none());
    if let Some(bc) = bc
        && c.is_relate_to_effect(duel, e)
        && bc.is_relate_to_battle(duel)
        && !bc.is_immune_to_effect(duel, e)
    {
        duel.overlay_full(c, bc, true)?;
    }
    Ok(())
}

fn dam300tg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let ct = a.e.get_handler(duel).get_overlay_count(duel);
    if chk == 0 {
        return Ok(ct > 0);
    }
    duel.set_target_player(1 - a.tp);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - a.tp, ct as i32 * 300);
    Ok(false)
}

fn dam300op(duel: &mut Duel, a: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let ct = a.e.get_handler(duel).get_overlay_count(duel);
    duel.damage(p, ct as u32 * 300, REASON_EFFECT)?;
    Ok(())
}

fn dam800cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let max = a.e.get_handler(duel).get_overlay_count(duel) as u16;
    Cost::detach_from_self_with(duel, a, chk, 1, max, |duel, e, og| e.set_label(duel, og.len() as i64))
}

fn dam800tg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let ct = a.e.get_label(duel) as i32;
    duel.set_target_player(1 - a.tp);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - a.tp, ct * 800);
    Ok(false)
}

fn dam800op(duel: &mut Duel, a: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let ct = a.e.get_label(duel) as u32;
    duel.damage(p, ct * 800, REASON_EFFECT)?;
    Ok(())
}
