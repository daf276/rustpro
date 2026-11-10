// Quickdraw Synchron
use crate::prelude::*;
use crate::testing::{Check, HAND};

const ID: u32 = 20932152;

// Two in hand to Special Summon by sending Psychic Kappa, a Normal Monster, from the hand.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (7892180, HAND), (7892180, HAND)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, spcon);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_CANNOT_BE_SYNCHRO_MATERIAL);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e2.set_value(
        duel,
        Value::Card(|duel, _, c| i32::from(!c.lists_archetype_as_material(duel, &[SET_SYNCHRON]))),
    );
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e3.set_code(duel, ID);
    c.register_effect(duel, e3);
}

fn spfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_able_to_grave_as_cost(duel)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_matching_card(spfilter, tp, LOCATION_HAND, 0, 1, Some(c.into()))
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, spfilter, tp, LOCATION_HAND, 0, 1, 1, a.c.map(Into::into))?;
    duel.send_to_grave(g, REASON_COST)?;
    Ok(())
}
