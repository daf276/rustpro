// Destiny Board
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, SZONE};

const ID: u32 = 94212438;

pub const LISTED_SERIES: &[u16] = &[SET_SPIRIT_MESSAGE];
pub const LISTED_NAMES: &[u32] = &[ID];
// Spirit Message cards in the Deck to place, and copies in the hand and set to be activated.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (31893528, DECK),
    (31893528, DECK),
    (67287533, DECK),
    (67287533, DECK),
    (94772232, DECK),
    (94772232, DECK),
    (30170981, DECK),
    (30170981, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

// s.extraop, which only Dark Sanctuary (not ported) reads, is left out.
pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_END_PHASE);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_code(duel, EVENT_PHASE + PHASE_END);
    e2.set_count_limit(duel, 1, 0);
    e2.set_label(duel, i64::from(ID));
    e2.set_condition(duel, |duel, a| {
        duel.is_turn_player(1 - a.tp) && a.e.get_handler(duel).get_flag_effect(duel, ID) < 4
    });
    e2.set_operation(duel, plop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_code(duel, EVENT_LEAVE_FIELD);
    e3.set_condition(duel, |duel, a| {
        let tp = a.tp;
        a.eg.unwrap_or_default().is_exists(duel, |c, duel| cfilter1(c, duel, tp), 1, None)
    });
    e3.set_operation(duel, tgop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e4.set_code(duel, EVENT_LEAVE_FIELD);
    e4.set_operation(duel, tgop);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e5.set_code(duel, EVENT_ADJUST);
    e5.set_range(duel, LOCATION_SZONE);
    e5.set_property(duel, EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_CANNOT_DISABLE);
    e5.set_operation(duel, winop);
    c.register_effect(duel, e5);
}

fn plfilter(c: CardId, duel: &Duel, code: u32) -> bool {
    c.is_code(duel, code) && !c.is_forbidden(duel)
}

fn plop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    unported!(duel, duel.is_player_affected_by_effect(tp, CARD_DARK_SANCTUARY).is_some());
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || duel.get_location_count(tp, LOCATION_SZONE) <= 0 {
        return Ok(());
    }
    let passcode = CARDS_SPIRIT_MESSAGE.get(c.get_flag_effect(duel, ID)).copied().unwrap_or(0);
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 1));
    let g = duel.select_matching_card(
        tp,
        |c, duel| plfilter(c, duel, passcode),
        tp,
        LOCATION_DECK | LOCATION_HAND,
        0,
        1,
        1,
        None,
    )?;
    if let Some(tc) = g.get_first()
        && duel.move_to_field(tc, tp, tp, LOCATION_SZONE, POS_FACEUP, true)?
    {
        c.register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD, 0, 0);
    }
    Ok(())
}

fn cfilter1(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_controler(duel, tp) && (c.is_code(duel, ID) || c.is_set_card(duel, &[SET_SPIRIT_MESSAGE]))
}

fn cfilter2(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && (c.is_code(duel, ID) || c.is_set_card(duel, &[SET_SPIRIT_MESSAGE]))
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(cfilter2, a.tp, LOCATION_ONFIELD, 0, None);
    duel.send_to_grave(g, REASON_EFFECT)?;
    Ok(())
}

fn cfilter3(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && CARDS_SPIRIT_MESSAGE.iter().any(|&code| c.is_code(duel, code))
}

fn winop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(cfilter3, tp, LOCATION_ONFIELD, 0, Some(a.e.get_handler(duel).into()));
    if g.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) == 4 {
        duel.win(tp, WIN_REASON_DESTINY_BOARD);
    }
    Ok(())
}
