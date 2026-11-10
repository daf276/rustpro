// Jack-In-The-Hand
use crate::prelude::*;
use crate::testing::{DECK, HAND};

const ID: u32 = 51697825;

// In the hand, with Level 1 Normal Monsters of different names in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (1929294, DECK),
    (1929294, DECK),
    (27288416, DECK),
    (27288416, DECK),
    (20240828, DECK),
    (20240828, DECK),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level(duel, &[1]) && c.is_monster(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let rvg = duel.get_matching_group(|c, duel| thfilter(c, duel) && c.is_able_to_hand(duel), a.tp, LOCATION_DECK, 0, None);
    if chk == 0 {
        return Ok(rvg.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) >= 3);
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, PLAYER_ALL, LOCATION_DECK as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let rvg = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
    let mut g = aux::select_unselect_group(duel, rvg, 3, 3, &aux::dncheck, tp, HINTMSG_CONFIRM)?;
    if g.len() != 3 {
        return Ok(());
    }
    duel.confirm_cards(1 - tp, g)?;
    duel.hint(HINT_SELECTMSG, 1 - tp, HINTMSG_ATOHAND);
    let Some(sg) = g.select(duel, 1 - tp, 1, 1, None)?.get_first() else {
        return Ok(());
    };
    to_hand_or_grave(duel, sg, 1 - tp)?;
    g.remove(sg);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let Some(tg) = g.select(duel, tp, 1, 1, None)?.get_first() else {
        return Ok(());
    };
    to_hand_or_grave(duel, tg, tp)?;
    duel.shuffle_deck(tp)
}

fn to_hand_or_grave(duel: &mut Duel, c: CardId, player: u8) -> Script<()> {
    if c.is_able_to_hand(duel) {
        duel.send_to_hand(c, player, REASON_EFFECT)?;
        duel.confirm_cards(1 - player, c)?;
        duel.shuffle_hand(player)
    } else {
        duel.send_to_grave(c, REASON_RULE).map(drop)
    }
}
