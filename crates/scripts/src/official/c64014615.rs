// Pot of Acquisitiveness
use crate::prelude::*;
use crate::testing::{OPPONENT, REMOVED};

const ID: u32 = 64014615;

// Banished monsters of both players to shuffle back.
pub const SUPPORT: &[(u32, u16)] = &[
    (CARD_DARK_MAGICIAN, REMOVED),
    (CARD_DARK_MAGICIAN, REMOVED),
    (CARD_DARK_MAGICIAN, REMOVED),
    (CARD_DARK_MAGICIAN, REMOVED | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TODECK + CATEGORY_DRAW);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_monster(duel) && c.is_able_to_deck(duel)
}

fn sfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_location(duel, LOCATION_DECK) && c.is_controler(duel, tp)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_REMOVED) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 1) && duel.is_existing_target(filter, tp, LOCATION_REMOVED, LOCATION_REMOVED, 3, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, filter, tp, LOCATION_REMOVED, LOCATION_REMOVED, 3, 3, None)?;
    let count = g.map_or(0, |g| g.len() as u8);
    duel.set_operation_info(0, CATEGORY_TODECK, g, count, 0, 0);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tg) = duel.get_chain_info::<Option<CardSet>>(0, CHAININFO_TARGET_CARDS) else {
        return Ok(());
    };
    if tg.filter_count(duel, |c, duel| c.is_relate_to_effect(duel, e), None) != 3 {
        return Ok(());
    }
    duel.send_to_deck(tg, PLAYER_NONE, SEQ_DECKTOP, REASON_EFFECT)?;
    let g = duel.get_operated_group();
    if g.is_exists(duel, |c, duel| sfilter(c, duel, tp), 1, None) {
        duel.shuffle_deck(tp)?;
    }
    if g.is_exists(duel, |c, duel| sfilter(c, duel, 1 - tp), 1, None) {
        duel.shuffle_deck(1 - tp)?;
    }
    let ct = g.filter_count(duel, |c, duel| c.is_location(duel, LOCATION_DECK | LOCATION_EXTRA), None);
    if ct == 3 {
        duel.break_effect()?;
        duel.draw(tp, 1, REASON_EFFECT)?;
    }
    Ok(())
}
