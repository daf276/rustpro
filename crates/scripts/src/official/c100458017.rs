// Raise Moon Elegance Carol
use super::c100458016::{draw_target, drcost, register_draw_summon, track_searches};
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 100458017;

// Copies to draw and to Normal Summon without Tributing.
pub const SUPPORT: &[(u32, u16)] = &[(ID, DECK), (ID, DECK), (ID, DECK), (ID, HAND), (ID, HAND), (ID, HAND)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    register_draw_summon::<0>(duel, c, ID);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e2.set_code(duel, EFFECT_SUMMON_PROC);
    e2.set_condition(duel, |duel, a| {
        let Some(c) = a.c else { return true };
        a.min == 0 && c.is_level_above(duel, 5) && duel.get_location_count(c.get_controler(duel), LOCATION_MZONE) > 0
    });
    e2.set_operation(duel, |duel, a| {
        let Some(c) = a.c else { return Ok(()) };
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_BASE_ATTACK);
        e1.set_value(duel, 700);
        e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD_DISABLE & !RESET_TOFIELD));
        c.register_effect(duel, e1);
        Ok(())
    });
    c.register_effect(duel, e2);
    let e3a = duel.create_effect(c);
    e3a.set_description(duel, aux::stringid(ID, 2));
    e3a.set_category(duel, CATEGORY_DRAW);
    e3a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3a.set_property(duel, EFFECT_FLAG_DELAY);
    e3a.set_code(duel, EVENT_SUMMON_SUCCESS);
    e3a.set_condition(duel, |duel, a| a.e.get_handler(duel).is_summon_location(duel, LOCATION_HAND));
    e3a.set_cost(duel, drcost::<ID, 3, { RESET_PHASE | PHASE_END }>);
    e3a.set_target(duel, draw_target);
    e3a.set_operation(duel, drop::<ID>);
    c.register_effect(duel, e3a);
    let e3b = e3a.clone_effect(duel);
    e3b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3b);
    track_searches::<ID, { RESET_PHASE | PHASE_END }>(duel, c);
}

// Draw 1, also no Special Summons from the Extra Deck this turn except LIGHT Xyz Monsters.
pub(crate) fn drop<const ID: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.draw(a.tp, 1, REASON_EFFECT)?;
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 4));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| {
        c.is_location(duel, LOCATION_EXTRA) && !(c.is_attribute(duel, ATTRIBUTE_LIGHT) && c.is_xyz_monster(duel))
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, a.tp);
    Ok(())
}
