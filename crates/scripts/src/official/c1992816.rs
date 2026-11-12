// Number 9: Dyson Sphere
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 1992816;
pub const XYZ_NUMBER: u32 = 9;

// Copies on the field, some given materials by Xyz Import (Sangan) and attacked by Black Luster Soldier (3000 ATK, which cannot be attached),
// others attacked bare, with monsters in the Graveyard to attach.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (14602126, HAND),
    (14602126, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (10000100, MZONE | OPPONENT),
    (26202165, GRAVE),
    (26202165, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(9);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_hint_timing(duel, 0, TIMING_BATTLE_PHASE);
    e1.set_condition(duel, atkcon);
    e1.set_cost(duel, atkcost);
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_BE_BATTLE_TARGET);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_condition(duel, olcon);
    e2.set_target(duel, oltg);
    e2.set_operation(duel, olop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, dacon);
    e3.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e3.set_operation(duel, daop);
    c.register_effect(duel, e3);
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    if duel.get_attacker().is_some_and(|at| at.is_status(duel, STATUS_ATTACK_CANCELED)) {
        return false;
    }
    let c = a.e.get_handler(duel);
    Some(c) == duel.get_attack_target() && c.get_overlay_count(duel) > 0
}

fn atkcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.get_flag_effect(duel, ID) == 0);
    }
    c.register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_DAMAGE, 0, 1);
    Ok(false)
}

fn atkop(duel: &mut Duel, _: &Args) -> Script<()> {
    duel.negate_attack()?;
    Ok(())
}

fn olcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.get_overlay_count(duel) == 0 && c.is_type(duel, TYPE_XYZ)
}

fn oltg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && chkc.is_monster(duel));
    }
    if chk == 0 {
        return Ok(
            e.get_handler(duel).is_type(duel, TYPE_XYZ) && duel.is_existing_target(CardId::is_monster, tp, LOCATION_GRAVE, 0, 2, None)
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g = duel.select_target(tp, CardId::is_monster, tp, LOCATION_GRAVE, 0, 2, 2, None)?;
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, g, 2, 0, 0);
    Ok(false)
}

fn olop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if c.is_relate_to_effect(duel, e) {
        let g = duel.get_target_cards(e).unwrap_or_default();
        if !g.is_empty() {
            duel.overlay(c, g)?;
        }
    }
    Ok(())
}

fn dafilter(c: CardId, duel: &Duel, atk: i32) -> bool {
    c.is_faceup(duel) && c.get_attack(duel) > atk
}

fn dacon(duel: &Duel, a: &Args) -> bool {
    let atk = a.e.get_handler(duel).get_attack(duel);
    duel.is_phase(PHASE_MAIN1) && duel.is_existing_matching_card(|c, duel| dafilter(c, duel, atk), a.tp, 0, LOCATION_MZONE, 1, None)
}

fn daop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, e) {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3205);
        e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT + EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_DIRECT_ATTACK);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e1);
    }
    Ok(())
}
