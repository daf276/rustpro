// Tornado Wall
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 18605135;

pub const LISTED_NAMES: &[u32] = &[CARD_UMI];
// In the hand with Umi, and Elfnote Lucina to attack.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (CARD_UMI, HAND),
    (CARD_UMI, HAND),
    (13597785, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_condition(duel, umicheck);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_AVOID_BATTLE_DAMAGE);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_target_range(duel, 1, 0);
    e2.set_condition(duel, umicheck);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_code(duel, EFFECT_SELF_DESTROY);
    e3.set_condition(duel, |duel, a| !umicheck(duel, a));
    c.register_effect(duel, e3);
}

fn umicheck(duel: &Duel, _: &Args) -> bool {
    duel.is_environment(CARD_UMI)
}
