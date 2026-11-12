// Ally of Justice Light Gazer
use super::c44035031::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};
use engine::constants::MSG_ATTACK;

const ID: u32 = 19204398;

// In the Extra Deck, Synchro Summoned with Angel Trumpeter (a Level 4 Tuner) and Kojikocy (Level 4), with Key Mace (LIGHT) in the
// opponent's Graveyard for the ATK.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (87979586, MZONE),
    (87979586, MZONE),
    (1184620, MZONE),
    (1184620, MZONE),
    (1929294, GRAVE | OPPONENT),
    (1929294, GRAVE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::Card(val));
    c.register_effect(duel, e1);
}

fn val(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    duel.get_matching_group_count(
        |c, duel| c.is_attribute(duel, ATTRIBUTE_LIGHT),
        c.get_controler(duel),
        0,
        LOCATION_GRAVE,
        None,
    ) as i32
        * 200
}
