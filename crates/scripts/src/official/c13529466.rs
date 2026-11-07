// Invoked Caliga
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 13529466;
const ALEISTER_THE_INVOKER: u32 = 86120751;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Code(ALEISTER_THE_INVOKER),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_attribute_full(duel, ATTRIBUTE_DARK, Some(fc), sumtype, tp)),
    ];
}

// On the field for both players' monster effects (Elfnote Lucina) and an attack into the opponent's monsters.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (13597785, HAND),
    (13597785, HAND | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_FIELD);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_operation(duel, aclimit1);
    c.register_effect(duel, e1);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, 1, 0);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).has_flag_effect(duel, ID));
    e3.set_value(duel, Value::EffectPlayer(elimit));
    c.register_effect(duel, e3);
    let e4 = e1.clone_effect(duel);
    e4.set_operation(duel, aclimit3);
    c.register_effect(duel, e4);
    let e6 = e3.clone_effect(duel);
    e6.set_condition(duel, |duel, a| a.e.get_handler(duel).has_flag_effect(duel, ID + 1));
    e6.set_target_range(duel, 0, 1);
    c.register_effect(duel, e6);
    let e7 = duel.create_effect(c);
    e7.set_type(duel, EFFECT_TYPE_FIELD);
    e7.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e7.set_code(duel, EFFECT_CANNOT_ATTACK_ANNOUNCE);
    e7.set_range(duel, LOCATION_MZONE);
    e7.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e7.set_condition(duel, |duel, a| a.e.get_handler(duel).has_flag_effect(duel, ID + 2));
    e7.set_target_card(duel, |duel, e, c| i64::from(c.get_field_id(duel)) != e.get_label(duel));
    c.register_effect(duel, e7);
    let e8 = duel.create_effect(c);
    e8.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e8.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e8.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e8.set_range(duel, LOCATION_MZONE);
    e8.set_operation(duel, checkop);
    e8.set_label_object(duel, Some(e7));
    c.register_effect(duel, e8);
}

const RESET_TURN_END: u32 = RESET_EVENT | RESETS_STANDARD_DISABLE | RESET_CONTROL | RESET_PHASE | PHASE_END;

fn aclimit1(duel: &mut Duel, a: &Args) -> Script<()> {
    if a.ep == a.tp && a.re.is_some_and(|re| re.is_monster_effect(duel)) {
        a.e.get_handler(duel).register_flag_effect(duel, ID, RESET_TURN_END, 0, 1);
    }
    Ok(())
}

fn aclimit3(duel: &mut Duel, a: &Args) -> Script<()> {
    if a.ep != a.tp && a.re.is_some_and(|re| re.is_monster_effect(duel)) {
        a.e.get_handler(duel).register_flag_effect(duel, ID + 1, RESET_TURN_END, 0, 1);
    }
    Ok(())
}

fn elimit(duel: &Duel, _: EffectId, re: EffectId, _: u8) -> bool {
    re.is_monster_effect(duel)
}

fn checkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.has_flag_effect(duel, ID + 2) {
        return Ok(());
    }
    let fid = a.eg.and_then(|eg| eg.get_first()).map_or(0, |tc| tc.get_field_id(duel));
    c.register_flag_effect(duel, ID + 2, RESETS_STANDARD_PHASE_END, 0, 1);
    if let Some(e7) = a.e.get_label_object(duel) {
        e7.set_label(duel, i64::from(fid));
    }
    Ok(())
}
