// Magical Marionette
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 8034697;

// On the field, with Spellbook Organization in the hand to activate for Spell Counters.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (96677818, HAND), (96677818, HAND), (96677818, HAND), (96677818, HAND)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_counter_permit(duel, COUNTER_SPELL);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_FIELD);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e0.set_code(duel, EVENT_CHAINING);
    e0.set_range(duel, LOCATION_MZONE);
    e0.set_operation(duel, aux::chainreg);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_FIELD);
    e1.set_code(duel, EVENT_CHAIN_SOLVING);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_operation(duel, acop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_value(duel, Value::Card(|duel, _, c| i32::from(c.get_counter(duel, COUNTER_SPELL)) * 200));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_cost(duel, descost);
    e3.set_target(duel, destarg);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
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

fn descost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.is_can_remove_counter(duel, tp, COUNTER_SPELL, 2, REASON_COST));
    }
    c.remove_counter(duel, tp, COUNTER_SPELL, 2, REASON_COST)?;
    Ok(false)
}

fn destarg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
