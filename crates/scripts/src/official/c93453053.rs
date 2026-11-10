// Maliss in the Mirror
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 93453053;

// In the hand, Graveyard and Deck, with Maliss <P> March Hare on the field and Dormouse in the Deck, Sangan on the opponent's field to negate,
// and Maliss in Underground in the hand to banish this card from the Graveyard for the Spell in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (ID, GRAVE),
    (ID, DECK),
    (68337209, HAND),
    (68337209, HAND),
    (20938824, MZONE),
    (32061192, DECK),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE + CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, rmdistg);
    e1.set_operation(duel, rmdisop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_REMOVE + CATEGORY_SEARCH + CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_REMOVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, rmthtg);
    e2.set_operation(duel, rmthop);
    c.register_effect(duel, e2);
}

fn rmdisfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MALISS])
        && c.is_monster(duel)
        && (c.is_faceup(duel) || c.is_location(duel, LOCATION_HAND))
        && c.is_able_to_remove(duel)
}

fn rmdistg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && chkc.is_negatable_monster(duel));
    }
    if chk == 0 {
        return Ok(
            duel.is_existing_target(CardId::is_negatable_monster, tp, 0, LOCATION_MZONE, 1, None)
                && duel.is_existing_matching_card(rmdisfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, None),
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
    let g = duel.select_target(tp, CardId::is_negatable_monster, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, tp, (LOCATION_HAND | LOCATION_MZONE) as i32);
    duel.set_operation_info(0, CATEGORY_DISABLE, g, 1, tp, 0);
    Ok(false)
}

fn rmdisop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tc = duel.get_first_target();
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, rmdisfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first()
        && duel.remove(g, POS_FACEUP, REASON_EFFECT)? > 0
        && sc.is_location(duel, LOCATION_REMOVED)
        && let Some(tc) = tc
        && tc.is_relate_to_effect(duel, e)
        && tc.is_faceup(duel)
    {
        tc.negate_effects(duel, e.get_handler(duel), RESET_PHASE | PHASE_END, false);
    }
    Ok(())
}

fn rmthfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    let typ = c.get_main_card_type(duel);
    c.is_set_card(duel, &[SET_MALISS])
        && c.is_able_to_remove(duel)
        && duel.is_existing_matching_card(|tc, duel| thfilter(tc, duel, typ), tp, LOCATION_DECK, 0, 1, None)
}

fn thfilter(c: CardId, duel: &Duel, typ: u32) -> bool {
    c.is_set_card(duel, &[SET_MALISS]) && c.is_type(duel, typ) && c.is_able_to_hand(duel)
}

fn rmthtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && rmthfilter(chkc, duel, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| rmthfilter(c, duel, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, |c, duel| rmthfilter(c, duel, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn rmthop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, e) || duel.remove(tc, POS_FACEUP, REASON_EFFECT)? == 0 || !tc.is_location(duel, LOCATION_REMOVED) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let typ = tc.get_main_card_type(duel);
    let g = duel.select_matching_card(tp, |c, duel| thfilter(c, duel, typ), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
