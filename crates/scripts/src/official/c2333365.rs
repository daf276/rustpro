// Tyr of the Nordic Champions
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 2333365;

// On the field with Valkyrie of the Nordic Ascendant, protected from Elfnote Lucina's attacks, and in the hand as a
// Level 4 material with Garmr for Fabled Levilazebul, with Mara of the Nordic Alfar on the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, HAND),
    (ID, HAND),
    (40844552, MZONE),
    (13597785, MZONE | OPPONENT),
    (73417207, MZONE),
    (88283496, HAND),
    (88283496, HAND),
    (21281085, EXTRA),
    (21281085, EXTRA),
    (21281085, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK), Check::MaterialFrom(HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_SELF_DESTROY);
    e1.set_condition(duel, descon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, 0, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_CANNOT_SELECT_BATTLE_TARGET);
    e2.set_value(duel, Value::Card(atlimit));
    c.register_effect(duel, e2);
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    !duel.is_existing_matching_card(
        aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_NORDIC])),
        0,
        LOCATION_MZONE,
        LOCATION_MZONE,
        1,
        Some(c.into()),
    )
}

fn atlimit(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    i32::from(c.is_faceup(duel) && !c.is_code(duel, ID) && c.is_set_card(duel, &[SET_NORDIC]))
}
