// Cyber Repair Plant
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND};

const ID: u32 = 86686671;

pub const LISTED_NAMES: &[u32] = &[CARD_CYBER_DRAGON];
// Cyber Dragons in the Graveyard for the condition and to shuffle, three for both effects, and more in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (CARD_CYBER_DRAGON, GRAVE),
    (CARD_CYBER_DRAGON, GRAVE),
    (CARD_CYBER_DRAGON, GRAVE),
    (CARD_CYBER_DRAGON, DECK),
    (CARD_CYBER_DRAGON, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(|c, duel| c.is_code(duel, CARD_CYBER_DRAGON), a.tp, LOCATION_GRAVE, 0, 1, None)
    });
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_MACHINE) && c.is_attribute(duel, ATTRIBUTE_LIGHT) && c.is_able_to_hand(duel)
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_MACHINE) && c.is_attribute(duel, ATTRIBUTE_LIGHT) && c.is_able_to_deck(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(e.get_label(duel) != 0 && chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && tdfilter(chkc, duel));
    }
    let b1 = duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None);
    let b2 = duel.is_existing_target(tdfilter, tp, LOCATION_GRAVE, 0, 1, None);
    let b3 = b1 && b2 && duel.is_existing_matching_card(|c, duel| c.is_code(duel, CARD_CYBER_DRAGON), tp, LOCATION_GRAVE, 0, 3, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(
        tp,
        &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2)), (b3, aux::stringid(ID, 3))],
    )?;
    let op = op.map_or(0, |op| op as i64);
    e.set_label(duel, op);
    let mut cat = 0;
    if op == 1 || op == 3 {
        cat = CATEGORY_TOHAND + CATEGORY_SEARCH;
        e.set_category(duel, cat);
        e.set_property(duel, 0);
        duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    }
    if op == 2 || op == 3 {
        e.set_category(duel, cat | CATEGORY_TODECK);
        e.set_property(duel, EFFECT_FLAG_CARD_TARGET);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
        let g = duel.select_target(tp, tdfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
        duel.set_operation_info(0, CATEGORY_TODECK, g, 1, tp, 0);
    }
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let op = e.get_label(duel);
    let mut breakeff = false;
    if op == 1 || op == 3 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, g)?;
            breakeff = true;
        }
    }
    if (op == 2 || op == 3)
        && let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
    {
        if breakeff {
            duel.break_effect()?;
        }
        duel.send_to_deck(tc, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    }
    Ok(())
}
