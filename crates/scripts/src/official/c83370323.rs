// Torapart
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 83370323;

// On the field as the Tuner of a Synchro Summon of Stardust Charge Warrior (Warrior, Level 6) with Flying Penguin (Level 4).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (5628232, MZONE),
    (5628232, MZONE),
    (64880894, EXTRA),
    (64880894, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::MaterialFrom(MZONE), Check::SpecialSummonedCode(64880894)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_BE_MATERIAL);
    e1.set_condition(duel, |_, a| a.r == REASON_SYNCHRO);
    e1.set_operation(duel, op);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_CANNOT_BE_SYNCHRO_MATERIAL);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e2.set_value(duel, Value::Card(|duel, _, c| i32::from(!c.is_race(duel, RACE_WARRIOR))));
    c.register_effect(duel, e2);
}

fn op(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(sync) = c.get_reason_card(duel) else { return Ok(()) };
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e1.set_operation(duel, atkop);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    sync.register_effect(duel, e1);
    Ok(())
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_target_range(duel, 0, 1);
    e1.set_value(duel, Value::EffectPlayer(aclimit));
    e1.set_reset(duel, RESET_PHASE | PHASE_DAMAGE);
    duel.register_effect(e1, a.tp);
    Ok(())
}

fn aclimit(duel: &Duel, _: EffectId, re: EffectId, _: u8) -> bool {
    re.get_handler(duel).is_trap(duel) && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)
}
