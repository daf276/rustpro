// Tatsunecro
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};
use engine::constants::MSG_ATTACK;

const ID: u32 = 3096468;

// Copies in the hand to Normal Summon and on the field to attack, with Plaguespreader Zombie, The 13th Grave (hand) and Revived King Ha Des for
// the hand Synchro Summon. Its effects are continuous, and the hand material it grants is to other cards.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (33420078, MZONE),
    (32864, HAND),
    (5309481, EXTRA),
    (5309481, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

struct HandSynchro;

impl HandMaterial for HandSynchro {
    const ID: u32 = ID;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| !c.is_race(duel, RACE_ZOMBIE));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_HAND_SYNCHRO);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e2.set_label(duel, ID.into());
    e2.set_condition(duel, syncon);
    e2.set_value(duel, Value::HandSynchro(synval));
    c.register_effect(duel, e2);
}

fn syncon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_normal_summoned(duel)
}

fn synval(duel: &mut Duel, e: EffectId, c: CardId, _: CardId) -> bool {
    if !c.is_location(duel, LOCATION_HAND) {
        return false;
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_HAND_SYNCHRO + EFFECT_SYNCHRO_CHECK);
    e1.set_label(duel, ID.into());
    e1.set_synchro_check_target(duel, Synchro::hand_material_check::<HandSynchro>);
    c.register_effect(duel, e1);
    c.register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD, 0, 1);
    true
}
