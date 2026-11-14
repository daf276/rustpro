// Warlock of the Ice Barrier
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};
use engine::constants::MSG_ATTACK;

const ID: u32 = 18482591;

pub const LISTED_SERIES: &[u16] = &[SET_ICE_BARRIER];

// Copies and Ice Barrier monsters on the field, with Spells in the hand to stop.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (23950192, MZONE),
    (90311614, MZONE),
    (83764718, HAND),
    (83764718, HAND),
];
// Only continuous effects.
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, 1, 1);
    e2.set_condition(duel, con);
    e2.set_value(duel, Value::EffectPlayer(aclimit));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EVENT_SSET);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, con);
    e3.set_operation(duel, aclimset);
    c.register_effect(duel, e3);
}

pub fn con(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    duel.is_existing_matching_card(
        aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_ICE_BARRIER])),
        c.get_controler(duel),
        LOCATION_MZONE,
        0,
        1,
        Some(c.into()),
    )
}

fn aclimit(duel: &Duel, _: EffectId, re: EffectId, _: u8) -> bool {
    if !re.is_has_type(duel, EFFECT_TYPE_ACTIVATE) || !re.is_spell_effect(duel) {
        return false;
    }
    let c = re.get_handler(duel);
    !c.is_location(duel, LOCATION_SZONE) || c.get_flag_effect(duel, ID) > 0
}

fn aclimset(duel: &mut Duel, a: &Args) -> Script<()> {
    for tc in a.eg.unwrap_or_default() {
        tc.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END | RESET_OPPO_TURN, 0, 1);
    }
    Ok(())
}
