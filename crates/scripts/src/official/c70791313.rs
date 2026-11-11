// Royal Magical Library
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 70791313;

// On the field, with Spellbook Organization in the hand to activate for Spell Counters.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (96677818, HAND), (96677818, HAND), (96677818, HAND), (96677818, HAND)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_counter_permit(duel, COUNTER_SPELL);
    c.set_counter_limit(duel, COUNTER_SPELL, 3);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_FIELD);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e0.set_code(duel, EVENT_CHAINING);
    e0.set_range(duel, LOCATION_MZONE);
    e0.set_operation(duel, aux::chainreg);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_FIELD);
    e1.set_code(duel, EVENT_CHAIN_SOLVED);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_operation(duel, acop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_cost(duel, drcost);
    e2.set_target(duel, stock::draw_target::<1>);
    e2.set_operation(duel, stock::draw);
    c.register_effect(duel, e2);
}

fn acop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if a.re
        .is_some_and(|re| re.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && re.is_spell_effect(duel))
        && c.get_flag_effect(duel, 1) > 0
    {
        c.add_counter(duel, COUNTER_SPELL, 1);
    }
    Ok(())
}

fn drcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_can_remove_counter(duel, a.tp, COUNTER_SPELL, 3, REASON_COST));
    }
    c.remove_counter(duel, a.tp, COUNTER_SPELL, 3, REASON_COST)?;
    Ok(false)
}
