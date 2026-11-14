// Frightfur Patchwork
use crate::prelude::*;
use crate::testing::{DECK, HAND};

const ID: u32 = 34773082;

pub const LISTED_NAMES: &[u32] = &[CARD_POLYMERIZATION];
pub const LISTED_SERIES: &[u16] = &[SET_EDGE_IMP];
// In the hand, with Polymerization and Edge Imp Chain in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (24094653, DECK), (24094653, DECK), (61173621, DECK), (61173621, DECK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn filter1(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_POLYMERIZATION) && c.is_able_to_hand(duel)
}

fn filter2(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_EDGE_IMP]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter1, tp, LOCATION_DECK, 0, 1, None)
            && duel.is_existing_matching_card(filter2, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 2, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g1 = duel.get_matching_group(filter1, tp, LOCATION_DECK, 0, None);
    let g2 = duel.get_matching_group(filter2, tp, LOCATION_DECK, 0, None);
    if !g1.is_empty() && !g2.is_empty() {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let sg1 = g1.select(duel, tp, 1, 1, None)?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let sg2 = g2.select(duel, tp, 1, 1, None)?;
        let sg = sg1.union(sg2);
        duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, sg)?;
    }
    Ok(())
}
