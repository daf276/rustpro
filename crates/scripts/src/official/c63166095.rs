// Sky Striker Mobilize - Engage!
use crate::prelude::*;
use crate::testing::{DECK, GRAVE, HAND};

const ID: u32 = 63166095;

pub const LISTED_NAMES: &[u32] = &[ID];
// In the hand with Sky Striker cards in the Deck to search and Spells in the GY to enable the draw.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (98338152, DECK),
    (52340444, DECK),
    (98338152, GRAVE),
    (52340444, GRAVE),
    (52340444, GRAVE),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.get_sequence(duel) < 5
}

fn condition(duel: &Duel, a: &Args) -> bool {
    !duel.is_existing_matching_card(cfilter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SKY_STRIKER]) && c.is_able_to_hand(duel) && !c.is_code(duel, ID)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_DRAW, None, 1, tp, 1);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
        if duel.is_player_can_draw(tp, 1)
            && duel.get_matching_group_count(CardId::is_spell, tp, LOCATION_GRAVE, 0, None) >= 3
            && duel.select_yes_no(tp, aux::stringid(ID, 0))?
        {
            duel.break_effect()?;
            duel.shuffle_deck(tp)?;
            duel.draw(tp, 1, REASON_EFFECT)?;
        }
    }
    Ok(())
}
