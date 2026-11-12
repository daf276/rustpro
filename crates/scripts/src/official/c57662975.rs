// Darkstorm Dragon
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 57662975;

// On the field and in the hand, with Kojikocys to Tribute for a Gemini Summon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, HAND), (ID, HAND), (1184620, MZONE), (1184620, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Gemini::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, Gemini::effect_status_condition);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_spell_trap(duel) && c.is_able_to_grave_as_cost(duel)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::send_to_grave_cost(duel, a, chk, cfilter, LOCATION_ONFIELD)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let g = duel.get_matching_group(|c, duel| c.is_spell_trap(duel), a.tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(|c, duel| c.is_spell_trap(duel), a.tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
