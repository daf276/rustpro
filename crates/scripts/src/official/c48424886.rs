// El Shaddoll Grysta
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND};

const ID: u32 = 48424886;

pub const LISTED_SERIES: &[u16] = &[SET_SHADDOLL];
pub const MATERIAL_SETCODE: &[u16] = &[SET_SHADDOLL];

// In the Extra Deck, Fusion Summoned by Polymerization with Shaddoll Squamata and Charcoal Inpachi (FIRE), with more Shaddoll
// monsters in the hand to send, and sent to the GY by Dark Hole drawn later with Shaddoll Fusion there to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (24094653, HAND),
    (30328508, HAND),
    (13179332, HAND),
    (77723643, HAND),
    (53129443, DECK),
    (53129443, DECK),
    (44394295, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    super::c94977269::add_procedure::<ATTRIBUTE_FIRE>(duel, c);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_DISABLE_SUMMON + CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_code(duel, EVENT_SPSUMMON);
    e3.set_count_limit(duel, 1, ID);
    e3.set_condition(duel, |duel, a| a.tp != a.ep && duel.get_current_chain() == 0);
    e3.set_target(duel, target);
    e3.set_operation(duel, operation);
    c.register_effect(duel, e3);
    super::c94977269::register_to_hand(duel, c, aux::stringid(ID, 1));
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SHADDOLL])
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, a.tp, LOCATION_HAND, 0, 1, None));
    }
    let count = a.eg.map_or(0, |eg| eg.len()) as u8;
    duel.set_operation_info(0, CATEGORY_DISABLE_SUMMON, a.eg, count, 0, 0);
    duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, count, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let eg = a.eg.unwrap_or_default();
    duel.negate_summon(eg);
    duel.destroy(eg, REASON_EFFECT)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.break_effect()?;
        duel.send_to_grave(g, REASON_EFFECT)?;
    }
    Ok(())
}
