// Vanadis of the Nordic Ascendant
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE};

const ID: u32 = 61777313;

pub const LISTED_SERIES: &[u16] = &[SET_NORDIC];
// On the field with Nordic monsters in the Deck to send, and as the Level 2 Tuner (after sending Valkyrie) with Garmr and Tyr for Fabled Levilazebul.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (88283496, MZONE),
    (2333365, MZONE),
    (40844552, DECK),
    (40844552, DECK),
    (77060848, DECK),
    (21281085, EXTRA),
    (21281085, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::MaterialFrom(MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, ID);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_SYNCHRO_MAT_RESTRICTION);
    e2.set_value(duel, Value::Card(|duel, _, c| i32::from(c.is_set_card(duel, &[SET_NORDIC]))));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_count_limit(duel, 1, 0);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_cost(duel, cost);
    e3.set_operation(duel, operation);
    c.register_effect(duel, e3);
}

fn cfilter(c: CardId, duel: &Duel, lv: u32) -> bool {
    let clv = c.get_level(duel);
    c.is_set_card(duel, &[SET_NORDIC]) && clv > 0 && clv != lv && c.is_able_to_grave_as_cost(duel)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let lv = e.get_handler(duel).get_level(duel);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| cfilter(c, duel, lv), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, |c, duel| cfilter(c, duel, lv), tp, LOCATION_DECK, 0, 1, 1, None)?;
    duel.send_to_grave(g, REASON_COST)?;
    let level = g.get_first().map_or(0, |tc| tc.get_level(duel));
    e.set_label(duel, level.into());
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if c.is_relate_to_effect(duel, e) && c.is_faceup(duel) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CHANGE_LEVEL);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        e1.set_value(duel, e.get_label(duel) as i32);
        c.register_effect(duel, e1);
    }
    Ok(())
}
