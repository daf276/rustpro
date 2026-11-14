// Beiige, Vanguard of Dark World
use super::c7623640::sptg;
use crate::prelude::*;
use crate::testing::{GRAVE, HAND};

const ID: u32 = 33731070;

// The Gates of Dark World on the field discards this card by its effect, banishing Zure, Knight of Dark World for its cost.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (ID, HAND), (33017655, 0), (7459013, GRAVE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).get_previous_location(duel) == LOCATION_HAND
            && a.r & (REASON_DISCARD | REASON_EFFECT) == REASON_DISCARD | REASON_EFFECT
    });
    e1.set_target(duel, sptg);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
}
