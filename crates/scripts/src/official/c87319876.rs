// Trust Guardian
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 87319876;

// On the field with Kojikocy (Level 4) and Kuibelt the Blade Dragon (Level 7) in the Extra Deck to Synchro Summon, which Elfnote Lucinas battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (1184620, MZONE),
    (1184620, MZONE),
    (87837090, EXTRA),
    (87837090, EXTRA),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::MaterialFrom(MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CANNOT_BE_SYNCHRO_MATERIAL);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_value(duel, Value::Card(|duel, _, c| i32::from(c.is_level_below(duel, 6))));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_BE_MATERIAL);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_IGNORE_IMMUNE);
    e2.set_condition(duel, |_, a| a.r == REASON_SYNCHRO);
    e2.set_operation(duel, cop);
    c.register_effect(duel, e2);
}

fn cop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let rc = c.get_reason_card(duel);
    unported!(duel, rc.is_none());
    let Some(rc) = rc else { return Ok(()) };
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_COUNT);
    e1.set_count_limit(duel, 1, 0);
    e1.set_value(duel, Value::ReasonMut(valcon));
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    rc.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e2.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EVENT_DAMAGE_STEP_END);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).get_flag_effect(duel, ID) != 0);
    e2.set_operation(duel, adop);
    e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    rc.register_effect(duel, e2);
    Ok(())
}

fn valcon(duel: &mut Duel, e: EffectId, _: Option<EffectId>, r: u32, _: u8) -> bool {
    if r & REASON_BATTLE == 0 {
        return false;
    }
    e.get_handler(duel)
        .register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_DAMAGE, 0, 1);
    true
}

fn adop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(a.e.get_owner(duel));
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, -400);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e2);
    Ok(())
}
