// Vanquish Soul, Start!
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 35550352;

pub const LISTED_SERIES: &[u16] = &[SET_VANQUISH_SOUL];
// Vanquish Soul monsters of several Attributes on the field, in the hand to reveal and in the Deck to add, Vanquish Soul - Continue?
// in the Deck to Set, and the opponent's Dark Hole to replace the destruction of.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, 0),
    (29302858, MZONE),
    (29280200, MZONE),
    (93156774, MZONE),
    (29302858, MZONE),
    (29280200, HAND),
    (93156774, DECK),
    (29302858, DECK),
    (9091064, DECK),
    (27345070, DECK),
    (60883493, DECK),
    (53330789, DECK),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_FZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EFFECT_DESTROY_REPLACE);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, repltg);
    e2.set_operation(duel, replop);
    e2.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(replfilter(c, duel, e.get_handler_player(duel)))),
    );
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SET);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_PHASE + PHASE_END);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_condition(duel, setcon);
    e3.set_target(duel, settg);
    e3.set_operation(duel, setop);
    c.register_effect(duel, e3);
}

fn thtgfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    let attr = c.get_attribute(duel);
    c.is_set_card(duel, &[SET_VANQUISH_SOUL])
        && c.is_faceup(duel)
        && duel.is_existing_matching_card(|c, duel| thfilter(c, duel, attr), tp, LOCATION_DECK, 0, 1, None)
}

fn thfilter(c: CardId, duel: &Duel, attr: u32) -> bool {
    c.is_set_card(duel, &[SET_VANQUISH_SOUL]) && c.is_monster(duel) && !c.is_attribute(duel, attr) && c.is_able_to_hand(duel)
}

// The Lua checks a global `chkc`, which is nil.
fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| thtgfilter(c, duel, tp), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, |c, duel| thtgfilter(c, duel, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, a.e) || tc.is_facedown(duel) {
        return Ok(());
    }
    let attr = tc.get_attribute(duel);
    stock::search(duel, a, |c, duel| thfilter(c, duel, attr), LOCATION_DECK)
}

fn replfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_VANQUISH_SOUL])
        && c.is_location(duel, LOCATION_MZONE)
        && c.is_faceup(duel)
        && c.is_controler(duel, tp)
        && !c.is_reason(duel, REASON_REPLACE)
}

fn rvfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_VANQUISH_SOUL]) && c.is_monster(duel) && !c.is_public(duel)
}

fn repltg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(a.eg.unwrap_or_default().is_exists(duel, |c, duel| replfilter(c, duel, tp), 1, None)
            && duel.is_existing_matching_card(rvfilter, tp, LOCATION_HAND, 0, 1, None));
    }
    if duel.select_effect_yes_no_full(tp, e.get_handler(duel), 96)? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
        let tg = duel.select_matching_card(tp, rvfilter, tp, LOCATION_HAND, 0, 1, 1, None)?;
        e.set_label_object_group(duel, tg.get_first().map(CardSet::from));
        return Ok(true);
    }
    Ok(false)
}

fn replop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_CARD, 1 - tp, u64::from(ID));
    let tc = a.e.get_label_object_group(duel).unwrap_or_default();
    duel.confirm_cards(1 - tp, tc)?;
    duel.shuffle_hand(tp)?;
    Ok(())
}

fn setcon(duel: &Duel, a: &Args) -> bool {
    let filter = aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_VANQUISH_SOUL]));
    duel.is_turn_player(a.tp) && duel.is_existing_matching_card(filter, a.tp, LOCATION_MZONE, 0, 2, None)
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_VANQUISH_SOUL]) && c.is_trap(duel) && c.is_ssetable(duel)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && duel.is_existing_matching_card(setfilter, a.tp, LOCATION_DECK, 0, 1, None))
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let g = duel.select_matching_card(tp, setfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.sset(tp, g)?;
    }
    Ok(())
}
