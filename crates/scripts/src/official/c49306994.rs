// White Veil
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};
use engine::constants::MSG_DAMAGE;

const ID: u32 = 49306994;

struct Procedure;

impl EquipProcedure for Procedure {}

// In the hand, with monsters to equip to and attack the opponent's Set and weak monsters with, and the opponent's Spells and Traps to
// negate and destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (21844576, MZONE),
    (21844576, MZONE),
    (30655537, MZONE | OPPONENT),
    (88279736, SZONE | OPPONENT),
    (88279736, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Message(MSG_DAMAGE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Equip::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_target_range(duel, 0, 1);
    e1.set_value(duel, Value::EffectPlayer(aclimit));
    e1.set_condition(duel, actcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e2.set_condition(duel, actcon);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_BATTLE_DESTROYING);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_condition(duel, descon);
    e3.set_target(duel, destg);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_SINGLE);
    e4.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e4.set_code(duel, EVENT_LEAVE_FIELD_P);
    e4.set_operation(duel, checkop);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e5.set_code(duel, EVENT_LEAVE_FIELD);
    e5.set_label_object(duel, Some(e4));
    e5.set_operation(duel, leave);
    c.register_effect(duel, e5);
    aux::double_snare_validity(duel, c, LOCATION_SZONE);
}

fn aclimit(duel: &Duel, e: EffectId, re: EffectId, _: u8) -> bool {
    re.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && !re.get_handler(duel).is_immune_to_effect(duel, e)
}

fn actcon(duel: &Duel, a: &Args) -> bool {
    let tc = a.e.get_handler(duel).get_equip_target(duel);
    duel.get_attacker() == tc || duel.get_attack_target() == tc
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let g = duel.get_matching_group(CardId::is_negatable_spell_trap, tp, 0, LOCATION_ONFIELD, Some(c.into()));
    for tc in g {
        for code in [EFFECT_DISABLE, EFFECT_DISABLE_EFFECT] {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, code);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_DAMAGE);
            tc.register_effect(duel, e1);
        }
        if tc.is_type(duel, TYPE_TRAPMONSTER) {
            let e3 = duel.create_effect(c);
            e3.set_type(duel, EFFECT_TYPE_SINGLE);
            e3.set_code(duel, EFFECT_DISABLE_TRAPMONSTER);
            e3.set_reset(duel, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_DAMAGE);
            tc.register_effect(duel, e3);
        }
    }
    Ok(())
}

fn descon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel)
        .get_equip_target(duel)
        .and_then(|tc| tc.get_battle_target(duel))
        .is_some_and(|bc| bc.is_previous_controler(duel, 1 - a.tp))
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_spell_trap, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    let sg = duel.get_matching_group(CardId::is_spell_trap, tp, 0, LOCATION_ONFIELD, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(sg), sg.len() as u8, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let sg = duel.get_matching_group(CardId::is_spell_trap, a.tp, 0, LOCATION_ONFIELD, None);
    duel.destroy(sg, REASON_EFFECT)?;
    Ok(())
}

fn checkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let disabled = a.e.get_handler(duel).is_disabled(duel);
    a.e.set_label(duel, disabled.into());
    Ok(())
}

fn leave(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if a.e.get_label_object(duel).is_some_and(|e4| e4.get_label(duel) == 0)
        && c.is_previous_controler(duel, tp)
        && c.is_previous_location(duel, LOCATION_SZONE)
        && c.is_previous_position(duel, POS_FACEUP)
    {
        duel.damage(tp, 3000, REASON_EFFECT)?;
    }
    Ok(())
}
