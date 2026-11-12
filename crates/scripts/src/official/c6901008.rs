// Phantom King Hydride
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 6901008;

// Used as the non-Tuner for a Level 4 Synchro Summon (Armory Arm, Toxic Boxfish) with another Tuner (Lunar Rabbit Omajinai).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (47643326, MZONE),
    (47643326, MZONE),
    (29071332, EXTRA),
    (29071332, EXTRA),
    (48882106, EXTRA),
    (48882106, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::MaterialFrom(MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_NONTUNER);
    c.register_effect(duel, e1);
}
