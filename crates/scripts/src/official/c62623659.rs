// Zexal Construction
use crate::prelude::*;
use crate::testing::{DECK, HAND};

const ID: u32 = 62623659;

pub const LISTED_SERIES: &[u16] = &[SET_ZEXAL, SET_ZW, SET_ZS, SET_RANK_UP_MAGIC, SET_RANK_DOWN_MAGIC];
// Two copies in the hand, with ZW - Ultimate Shield in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (18865703, DECK), (18865703, DECK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND + CATEGORY_TODECK);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_deck(duel) && !c.is_public(duel)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_hand(duel)
        && ((c.is_monster(duel) && (c.is_set_card(duel, &[SET_ZW]) || c.is_set_card(duel, &[SET_ZS])))
            || (c.is_spell(duel)
                && (c.is_set_card(duel, &[SET_ZEXAL])
                    || c.is_set_card(duel, &[SET_RANK_UP_MAGIC])
                    || c.is_set_card(duel, &[SET_RANK_DOWN_MAGIC])))
            || c.is_trap(duel) && c.is_set_card(duel, &[SET_ZEXAL]))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(
            duel.is_existing_matching_card(filter, tp, LOCATION_HAND, 0, 1, Some(e.get_handler(duel).into()))
                && duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None),
        );
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_TODECK, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
    let rg = duel.select_matching_card(tp, filter, tp, LOCATION_HAND, 0, 1, 1, None)?;
    if rg.is_empty() {
        return Ok(());
    }
    duel.confirm_cards(1 - tp, rg)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let tg = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !tg.is_empty() {
        duel.send_to_hand(tg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, tg)?;
        duel.break_effect()?;
        duel.send_to_deck(rg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    }
    Ok(())
}
