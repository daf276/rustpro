// Secret Village of the Spellcasters
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 68462976;

// In the hand, with Spellcaster Normal Monsters on one or both sides.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (27125110, MZONE),
    (15042735, MZONE | OPPONENT),
    (15042735, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_ADJUST);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_operation(duel, adjustop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_target_range(duel, 1, 1);
    e3.set_label(duel, 0);
    e3.set_value(duel, Value::EffectPlayer(actlimit));
    c.register_effect(duel, e3);
    e2.set_label_object(duel, Some(e3));
}

fn actlimit(duel: &Duel, e: EffectId, te: EffectId, tp: u8) -> bool {
    if !te.is_has_type(duel, EFFECT_TYPE_ACTIVATE) || !te.is_spell_effect(duel) {
        return false;
    }
    e.get_label(duel) == if tp == e.get_handler_player(duel) { 1 } else { 2 }
}

fn adjustop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let spellcaster = aux::faceup_filter(|c, duel| c.is_race(duel, RACE_SPELLCASTER));
    let b1 = duel.is_existing_matching_card(&spellcaster, tp, LOCATION_MZONE, 0, 1, None);
    let b2 = duel.is_existing_matching_card(&spellcaster, tp, 0, LOCATION_MZONE, 1, None);
    if let Some(te) = a.e.get_label_object(duel) {
        te.set_label(
            duel,
            if !b1 {
                1
            } else if !b2 {
                2
            } else {
                0
            },
        );
    }
    Ok(())
}
