// Assault Blackwing - Kusanagi the Gathering Storm
use super::c10515412::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};
use engine::constants::MSG_ATTACK;

const ID: u32 = 31114334;

pub const LISTED_SERIES: &[u16] = &[SET_BLACKWING];
// In the Extra Deck, Synchro Summoned from Blackwing - Shamal the Sandstorm (Level 4 Tuner) and Blackwing - Gram the Shining Star
// (Level 5 Synchro Monster), then attacking the opponent's Sangan.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (8571567, MZONE),
    (8571567, MZONE),
    (17377751, MZONE),
    (17377751, MZONE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_code(duel, EFFECT_MATERIAL_CHECK);
    e0.set_value(duel, Value::MaterialCheck(matcheck));
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e1.set_label_object(duel, Some(e0));
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_ATTACK_ALL);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_PIERCE);
    c.register_effect(duel, e3);
}

fn matcheck(duel: &mut Duel, e: EffectId, c: CardId) {
    let blackwing = c
        .get_material(duel)
        .is_exists(duel, |mc, duel| mc.is_set_card(duel, &[SET_BLACKWING]), 1, None);
    e.set_label(duel, i64::from(blackwing));
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let mg = c.get_material(duel);
    if mg.is_empty() {
        return Ok(());
    }
    if a.e.get_label_object(duel).is_some_and(|e0| e0.get_label(duel) == 1) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_ADD_TYPE);
        e1.set_value(duel, TYPE_TUNER as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e1);
    }
    if c.is_disabled(duel) {
        return Ok(());
    }
    let atk = mg
        .filter(duel, CardId::is_synchro_monster, None)
        .get_sum(duel, |mc, duel| i64::from(mc.get_base_attack(duel)));
    if atk == 0 {
        return Ok(());
    }
    c.update_attack(duel, atk as i32, Some(RESETS_STANDARD_DISABLE_PHASE_END), None, None);
    Ok(())
}
