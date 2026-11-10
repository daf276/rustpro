// Anubis the Last Judge
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 60411677;
const MAN_WITH_THE_MARK: u32 = 97522863;

pub const LISTED_NAMES: &[u32] = &[CARD_TEMPLE_OF_THE_KINGS, MAN_WITH_THE_MARK];
// In the hand, on the field and in the Graveyard, with Temple of the Kings and Traps (Waboku, Trap Hole, Bottomless Trap Hole) in the
// Graveyard to place on the Deck, The Man with the Mark in the Deck, and a Set Temple of the Kings for the opponent's Mystical Space Typhoon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (ID, GRAVE),
    (CARD_TEMPLE_OF_THE_KINGS, GRAVE),
    (12607053, GRAVE),
    (4206964, GRAVE),
    (29401950, GRAVE),
    (MAN_WITH_THE_MARK, DECK),
    (MAN_WITH_THE_MARK, DECK),
    (CARD_TEMPLE_OF_THE_KINGS, SZONE),
    (5318639, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    c.add_must_be_special_summoned(duel);
    let e0 = duel.create_effect(c);
    e0.set_description(duel, aux::stringid(ID, 0));
    e0.set_type(duel, EFFECT_TYPE_FIELD);
    e0.set_property(duel, EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_CANNOT_DISABLE);
    e0.set_code(duel, EFFECT_SPSUMMON_PROC);
    e0.set_range(duel, LOCATION_HAND | LOCATION_GRAVE);
    e0.set_condition(duel, spcon);
    e0.set_target(duel, sptg);
    e0.set_operation(duel, spop);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::self_to_grave);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 2));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, descon);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_TEMPLE_OF_THE_KINGS) || c.is_trap(duel)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = a.e.get_handler_player(duel);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return false;
    }
    let g = duel.get_matching_group(tdfilter, tp, LOCATION_GRAVE, 0, None);
    let rg = g.filter(duel, CardId::is_able_to_deck_as_cost, Some(c.into()));
    g.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) >= 3 && aux::select_unselect_group_check(duel, rg, 2, 2, &aux::dncheck)
}

fn sptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let mut rg = duel.get_matching_group(tdfilter, tp, LOCATION_GRAVE, 0, a.c.map(CardSet::from));
    rg.keep_matching(duel, CardId::is_able_to_deck_as_cost, None);
    let g = aux::select_unselect_group_full(duel, rg, 2, 2, &aux::dncheck, tp, HINTMSG_TODECK, None, true)?;
    if g.is_empty() {
        return Ok(false);
    }
    e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(g) = a.e.get_label_object_group(duel) else { return Ok(()) };
    duel.hint_selection(g);
    duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKBOTTOM, REASON_COST)?;
    duel.sort_deckbottom(a.tp, a.tp, 2)?;
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, MAN_WITH_THE_MARK) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}

fn desconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_previous_type_on_field(duel, TYPE_SPELL | TYPE_TRAP)
        && c.is_previous_location(duel, LOCATION_ONFIELD)
        && c.is_previous_controler(duel, tp)
        && c.is_reason(duel, REASON_EFFECT)
}

fn descon(duel: &Duel, a: &Args) -> bool {
    a.eg.unwrap_or_default()
        .is_exists(duel, |c, duel| desconfilter(c, duel, a.tp), 1, None)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, 1 - tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
