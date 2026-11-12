// Scarred Warrior
use super::c44035031::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};
use engine::constants::MSG_ATTACK;

const ID: u32 = 45298492;

// In the Extra Deck, Synchro Summoned with Scrap Synchron (a Level 1 Tuner) and Kojikocy (Level 4), with Warriors on both sides.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (16449363, MZONE),
    (16449363, MZONE),
    (1184620, MZONE),
    (1184620, MZONE),
    (1184620, MZONE | OPPONENT),
    (30090452, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_SELECT_BATTLE_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 0, LOCATION_MZONE);
    e1.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(c != e.get_handler(duel) && c.is_faceup(duel) && c.is_race(duel, RACE_WARRIOR))),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_COUNT);
    e2.set_count_limit(duel, 1, 0);
    e2.set_value(duel, Value::Reason(|_, _, _, r, _| r & REASON_BATTLE != 0));
    c.register_effect(duel, e2);
}
