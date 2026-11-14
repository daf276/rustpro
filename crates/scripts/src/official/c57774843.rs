// Judgment Dragon
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND};

const ID: u32 = 57774843;

pub const LISTED_SERIES: &[u16] = &[SET_LIGHTSWORN];
// In the hand and on the field, with the four ported Lightsworn monsters (Ryko, Lyla, Wulf, Jain) in the Graveyard to Summon it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (21502796, GRAVE),
    (22624373, GRAVE),
    (58996430, GRAVE),
    (96235275, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, spcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_cost(duel, Cost::pay_lp::<1000>);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e3.set_category(duel, CATEGORY_DECKDES);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_count_limit(duel, 1, 0);
    e3.set_code(duel, EVENT_PHASE + PHASE_END);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, |duel, a| a.tp == duel.get_turn_player());
    e3.set_target(duel, target2);
    e3.set_operation(duel, operation2);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE);
    e4.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e4.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    c.register_effect(duel, e4);
}

fn spfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_LIGHTSWORN]) && c.is_monster(duel)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return false;
    }
    let g = duel.get_matching_group(spfilter, tp, LOCATION_GRAVE, 0, None);
    g.get_class_count(duel, |c, duel| c.get_code(duel) as i64) > 3
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(c.into())));
    }
    let sg = duel.get_matching_group(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, Some(c.into()));
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(sg), sg.len() as u8, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let sg = duel.get_matching_group(aux::TRUE, a.tp, LOCATION_ONFIELD, LOCATION_ONFIELD, Some(c.into()));
    duel.destroy(sg, REASON_EFFECT)?;
    Ok(())
}

fn target2(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DECKDES, None, 0, a.tp, 4);
    Ok(false)
}

fn operation2(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.discard_deck(a.tp, 4, REASON_EFFECT)?;
    Ok(())
}
