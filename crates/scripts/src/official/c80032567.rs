// Jurrac Giganoto
use super::c65961683::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};
use engine::constants::MSG_ATTACK;

const ID: u32 = 80032567;

pub const LISTED_SERIES: &[u16] = &[SET_JURRAC];
// In the Extra Deck, Synchro Summoned with Jurrac Megalo (a Level 1 Tuner) and Frostosaurus (a Level 6 Dinosaur), with Jurrac
// monsters in the Graveyard for the ATK.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (93170499, MZONE),
    (93170499, MZONE),
    (6631034, MZONE),
    (6631034, MZONE),
    (93170499, GRAVE),
    (93170499, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_JURRAC]));
    e1.set_value(duel, Value::Card(val));
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_JURRAC]) && c.is_monster(duel)
}

fn val(duel: &Duel, e: EffectId, _: CardId) -> i32 {
    duel.get_matching_group_count(filter, e.get_handler_player(duel), LOCATION_GRAVE, 0, None) as i32 * 200
}
