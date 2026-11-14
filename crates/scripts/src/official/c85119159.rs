// Onomatopickup
use crate::prelude::*;
use crate::testing::{DECK, HAND, MZONE};

const ID: u32 = 85119159;

pub const LISTED_NAMES: &[u32] = &[];
pub const LISTED_SERIES: &[u16] = &[SET_ONOMAT, SET_GAGAGA, SET_GOGOGO, SET_DODODO, SET_ZUBABA];

// In hand with Onomatokage in the Deck to add, and Gagaga Girl (Level 3) beside Gagaga Ganbara Knight (Level 4) to change levels.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (55088578, DECK),
    (55088578, DECK),
    (48393693, MZONE),
    (48393693, MZONE),
    (9491461, MZONE),
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
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, lvtg);
    e2.set_operation(duel, lvop);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ONOMAT]) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if !e.get_handler(duel).is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let g = duel.get_matching_group(filter, tp, LOCATION_DECK, 0, None);
    if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 0))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let sg = g.select(duel, tp, 1, 1, None)?;
        duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, sg)?;
    }
    Ok(())
}

fn lvfilter1(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.has_level(duel)
        && (c.is_set_card(duel, &[SET_GAGAGA])
            || c.is_set_card(duel, &[SET_GOGOGO])
            || c.is_set_card(duel, &[SET_DODODO])
            || c.is_set_card(duel, &[SET_ZUBABA]))
        && duel.is_existing_matching_card(
            |d, duel| lvfilter2(d, duel, c.get_level(duel)),
            tp,
            LOCATION_MZONE,
            0,
            1,
            Some(c.into()),
        )
}

fn lvfilter2(c: CardId, duel: &Duel, lv: u32) -> bool {
    c.is_faceup(duel) && c.has_level(duel) && !c.is_level(duel, &[lv])
}

fn lvtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && lvfilter1(chkc, duel, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| lvfilter1(c, duel, tp), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, |c, duel| lvfilter1(c, duel, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    Ok(false)
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if !e.get_handler(duel).is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel) {
        let lv = tc.get_level(duel);
        let g = duel.get_matching_group(|c, duel| lvfilter2(c, duel, lv), tp, LOCATION_MZONE, 0, Some(tc.into()));
        for lc in g {
            let e1 = duel.create_effect(e.get_handler(duel));
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_CHANGE_LEVEL);
            e1.set_value(duel, lv as i32);
            e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
            lc.register_effect(duel, e1);
        }
    }
    Ok(())
}
