// Royal Penguins Garden
use crate::prelude::*;
use crate::testing::{DECK, HAND, MZONE, SZONE};

const ID: u32 = 80893872;

pub const LISTED_SERIES: &[u16] = &[SET_PENGUIN];
// Activated from the hand to search Flying Penguin and Bolt Penguin, and on the field with Flying Penguin to reduce and Raigeki
// to discard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (5628232, DECK),
    (5628232, DECK),
    (48531733, DECK),
    (5628232, HAND),
    (5628232, MZONE),
    (12580477, HAND),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_LVCHANGE + CATEGORY_HANDES);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_target(duel, lvtg);
    e2.set_operation(duel, lvop);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_PENGUIN]) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
    if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 0))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let sg = g.select(duel, tp, 1, 1, None)?;
        duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, sg)?;
    }
    Ok(())
}

fn lvfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_PENGUIN]) && c.is_level_above(duel, 2) && (c.is_faceup(duel) || c.is_location(duel, LOCATION_HAND))
}

fn discardable(c: CardId, duel: &Duel) -> bool {
    c.is_discardable_full(duel, REASON_EFFECT | REASON_DISCARD, None)
}

fn lvtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(
            duel.is_existing_matching_card(lvfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, None)
                && duel.is_existing_matching_card(discardable, tp, LOCATION_HAND, 0, 1, None),
        );
    }
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, tp, 1);
    Ok(false)
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
    let g = duel.select_matching_card(tp, lvfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, 1, None)?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    if tc.is_location(duel, LOCATION_MZONE) {
        duel.hint_selection(g);
    } else {
        duel.confirm_cards(1 - tp, g)?;
        duel.shuffle_hand(tp)?;
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_LEVEL);
    e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD & !RESET_TOFIELD) | RESET_PHASE | PHASE_END);
    e1.set_value(duel, -1);
    tc.register_effect(duel, e1);
    if duel.is_existing_matching_card(discardable, tp, LOCATION_HAND, 0, 1, None) {
        duel.break_effect()?;
        duel.discard_hand(tp, CardId::is_discardable, 1, 1, REASON_EFFECT | REASON_DISCARD)?;
    }
    Ok(())
}
