// Archfiend Strategy
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND};

const ID: u32 = 90764871;

pub const LISTED_SERIES: &[u16] = &[SET_ARCHFIEND];

// Archfiend Soldiers to banish and search, and Archfiend Eccentrick (a Pendulum) to add to the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (49881766, GRAVE),
    (49881766, DECK),
    (49881766, DECK),
    (57624336, DECK),
    (57624336, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_FZONE);
    e1.set_target_range(duel, LOCATION_EMZONE, LOCATION_EMZONE);
    e1.set_target_card(duel, |duel, _, c| c.is_race(duel, RACE_FIEND));
    e1.set_value(duel, 1000);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, efftg);
    e2.set_operation(duel, effop);
    c.register_effect(duel, e2);
}

fn costfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_race(duel, RACE_FIEND) || c.is_set_card(duel, &[SET_ARCHFIEND])) && c.is_able_to_remove_as_cost(duel)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ARCHFIEND]) && !c.is_field_spell(duel) && c.is_able_to_hand(duel)
}

fn tedfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ARCHFIEND]) && c.is_pendulum_monster(duel)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let b1 = duel.is_existing_matching_card(costfilter, tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, None)
        && duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None);
    let b2 = duel.is_existing_matching_card(tedfilter, tp, LOCATION_DECK, 0, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    match op {
        Some(1) => {
            e.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
            let g = duel.select_matching_card(tp, costfilter, tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, 1, None)?;
            duel.remove(g, POS_FACEUP, REASON_COST)?;
            duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
        }
        Some(2) => {
            e.set_category(duel, CATEGORY_TOEXTRA);
            duel.set_operation_info(0, CATEGORY_TOEXTRA, None, 1, tp, LOCATION_DECK as i32);
        }
        _ => {}
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    match e.get_label(duel) {
        1 => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
            let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
            if !g.is_empty() {
                duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
                duel.confirm_cards(1 - tp, g)?;
            }
        }
        2 => {
            duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 3));
            let g = duel.select_matching_card(tp, tedfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
            if !g.is_empty() {
                duel.send_to_extra_p(g, tp, REASON_EFFECT)?;
            }
        }
        _ => {}
    }
    Ok(())
}
