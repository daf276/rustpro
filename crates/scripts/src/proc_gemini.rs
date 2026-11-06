// Port of proc_gemini.lua: Gemini.AddProcedure and the conditions of a Gemini monster's two states.
use engine::{Args, CardId, Duel};

use crate::constant::*;

pub struct Gemini;

impl Gemini {
    pub fn add_procedure(duel: &mut Duel, c: CardId) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_GEMINI_SUMMONABLE);
        c.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE + EFFECT_FLAG_IGNORE_IMMUNE);
        e2.set_code(duel, EFFECT_ADD_TYPE);
        e2.set_range(duel, LOCATION_MZONE | LOCATION_GRAVE);
        e2.set_condition(duel, Gemini::normal_status_condition);
        e2.set_value(duel, TYPE_NORMAL as i32);
        c.register_effect(duel, e2);
        let e3 = e2.clone_effect(duel);
        e3.set_code(duel, EFFECT_REMOVE_TYPE);
        e3.set_value(duel, TYPE_EFFECT as i32);
        c.register_effect(duel, e3);
    }

    pub fn effect_status_condition(duel: &Duel, a: &Args) -> bool {
        let c = a.e.get_handler(duel);
        !c.is_disabled(duel) && c.is_gemini_status(duel)
    }

    pub fn normal_status_condition(duel: &Duel, a: &Args) -> bool {
        let c = a.e.get_handler(duel);
        c.is_faceup(duel) && !c.is_gemini_status(duel)
    }
}
