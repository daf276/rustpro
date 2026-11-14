// Skyscraper
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 63035430;

pub const LISTED_SERIES: &[u16] = &[SET_ELEMENTAL_HERO];

// s[0], shared by every copy's condition and value.
const BOOSTED: u64 = (ID as u64) << 16;

// A copy to activate, Elemental HERO Avian, Burstinatrix and Sparkman to attack, and the opponent's Luster Dragon to attack into.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (21844576, MZONE),
    (58932615, MZONE),
    (20721928, MZONE),
    (11091375, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e2.set_condition(duel, atkcon);
    e2.set_target_card(duel, atktg);
    e2.set_value(duel, Value::Card(atkval));
    c.register_effect(duel, e2);
}

fn atkcon(duel: &Duel, _: &Args) -> bool {
    duel.set_script_state(BOOSTED, 0);
    duel.is_phase(PHASE_DAMAGE_CAL) && duel.get_attack_target().is_some()
}

fn atktg(duel: &Duel, _: EffectId, c: CardId) -> bool {
    Some(c) == duel.get_attacker() && c.is_set_card(duel, &[SET_ELEMENTAL_HERO])
}

fn atkval(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    let d = duel.get_attack_target();
    if duel.get_script_state(BOOSTED) != 0 || d.is_some_and(|d| c.get_attack(duel) < d.get_attack(duel)) {
        duel.set_script_state(BOOSTED, 1);
        return 1000;
    }
    0
}
