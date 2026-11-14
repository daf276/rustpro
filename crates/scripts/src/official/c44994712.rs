// Predaplant Longinephila
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT, REMOVED};

const ID: u32 = 44994712;

pub const LISTED_SERIES: &[u16] = &[SET_PREDAP];
pub const COUNTER_PLACE_LIST: &[u16] = &[COUNTER_PREDATOR];

pub const LISTED_NAMES: &[u32] = &[CARD_POLYMERIZATION, ID];
// In the hand and Graveyard, Predaprimitive to search, Polymerization in the Graveyard and banishment to Set, and Elfnote
// Lucina in the opponent's monster zone to place a counter on.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (89176044, DECK),
    (89176044, DECK),
    (CARD_POLYMERIZATION, GRAVE),
    (CARD_POLYMERIZATION, REMOVED),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1a = duel.create_effect(c);
    e1a.set_description(duel, aux::stringid(ID, 0));
    e1a.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1a.set_property(duel, EFFECT_FLAG_DELAY);
    e1a.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1a.set_count_limit(duel, 1, (ID, 0));
    e1a.set_target(duel, thtg);
    e1a.set_operation(duel, thop);
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e1b);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, efftg);
    e2.set_operation(duel, effop);
    c.register_effect(duel, e2);
}

fn predapthfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_PREDAP]) && c.is_able_to_hand(duel) && !c.is_code(duel, ID)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, predapthfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, predapthfilter, LOCATION_DECK)
}

fn predctfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_can_add_counter(duel, COUNTER_PREDATOR, 1)
}

fn polysetfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_POLYMERIZATION) && c.is_faceup(duel) && c.is_ssetable(duel)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let b1 = duel.is_existing_matching_card(predctfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None);
    let b2 = duel.is_existing_matching_card(polysetfilter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?;
    a.e.set_label(duel, op.map_or(0, |op| op as i64));
    match op {
        Some(1) => {
            a.e.set_category(duel, CATEGORY_COUNTER);
            duel.set_operation_info(0, CATEGORY_COUNTER, None, 1, tp, i32::from(COUNTER_PREDATOR));
        }
        Some(2) => {
            a.e.set_category(duel, CATEGORY_SET);
            duel.set_possible_operation_info(0, CATEGORY_LEAVE_GRAVE, None, 1, tp, 0);
        }
        _ => {}
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    match a.e.get_label(duel) {
        1 => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_COUNTER);
            let g = duel.select_matching_card(tp, predctfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
            let Some(tc) = g.get_first() else { return Ok(()) };
            duel.hint_selection(tc);
            if tc.add_counter(duel, COUNTER_PREDATOR, 1) && tc.is_level_above(duel, 2) {
                let e1 = duel.create_effect(a.e.get_handler(duel));
                e1.set_type(duel, EFFECT_TYPE_SINGLE);
                e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
                e1.set_code(duel, EFFECT_CHANGE_LEVEL);
                e1.set_condition(duel, |duel, a| a.e.get_handler(duel).has_counter(duel, COUNTER_PREDATOR));
                e1.set_value(duel, 1);
                e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
                tc.register_effect(duel, e1);
            }
        }
        2 => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
            let g = duel.select_matching_card(
                tp,
                aux::necro_valley_filter(polysetfilter),
                tp,
                LOCATION_GRAVE | LOCATION_REMOVED,
                0,
                1,
                1,
                None,
            )?;
            if !g.is_empty() {
                duel.hint_selection(g);
                duel.sset(tp, g)?;
            }
        }
        _ => {}
    }
    Ok(())
}
