// Gemini Lancer
use crate::prelude::*;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Gemini::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_PIERCE);
    e1.set_condition(duel, Gemini::effect_status_condition);
    c.register_effect(duel, e1);
}
