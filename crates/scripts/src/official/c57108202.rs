// Morphtronic Remoten
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 57108202;

pub const LISTED_SERIES: &[u16] = &[SET_MORPHTRONIC];

const SCOPEN: u32 = 10591919;

// On the field in both positions, with Level 3 Morphtronic Scopen in the hand, Deck and Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (SCOPEN, HAND),
    (SCOPEN, HAND),
    (SCOPEN, DECK),
    (SCOPEN, DECK),
    (SCOPEN, GRAVE),
    (ID, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE + CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, cona);
    e1.set_target(duel, tga);
    e1.set_operation(duel, opa);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_count_limit(duel, 1, 0);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, cond);
    e2.set_target(duel, tgd);
    e2.set_operation(duel, opd);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel, lv: u32) -> bool {
    c.is_set_card(duel, &[SET_MORPHTRONIC]) && c.get_level(duel) == lv && c.is_able_to_hand(duel)
}

fn cona(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    !c.is_disabled(duel) && c.is_attack_pos(duel)
}

fn filtera(c: CardId, duel: &Duel, tp: u8) -> bool {
    let lv = c.get_level(duel);
    c.is_set_card(duel, &[SET_MORPHTRONIC])
        && lv > 0
        && c.is_able_to_remove(duel)
        && aux::sp_elim_filter(c, duel, true, false)
        && duel.is_existing_matching_card(|c, duel| filter(c, duel, lv), tp, LOCATION_DECK, 0, 1, None)
}

fn tga(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let location = LOCATION_MZONE | LOCATION_GRAVE;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, location) && chkc.is_controler(duel, tp) && filtera(chkc, duel, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| filtera(c, duel, tp), tp, location, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, |c, duel| filtera(c, duel, tp), tp, location, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn opa(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && duel.remove(tc, POS_FACEUP, REASON_EFFECT)? != 0
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let lv = tc.get_level(duel);
        let g = duel.select_matching_card(tp, |c, duel| filter(c, duel, lv), tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, g)?;
        }
    }
    Ok(())
}

fn cond(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    !c.is_disabled(duel) && c.is_defense_pos(duel)
}

fn filterd(c: CardId, duel: &Duel, tp: u8) -> bool {
    let lv = c.get_level(duel);
    c.is_set_card(duel, &[SET_MORPHTRONIC])
        && lv > 0
        && c.is_able_to_grave(duel)
        && duel.is_existing_matching_card(|c, duel| filter(c, duel, lv), tp, LOCATION_GRAVE, 0, 1, None)
}

fn tgd(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| filterd(c, duel, tp), tp, LOCATION_HAND, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_HAND as i32);
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn opd(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, |c, duel| filterd(c, duel, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    duel.send_to_grave(tc, REASON_EFFECT)?;
    if !tc.is_location(duel, LOCATION_GRAVE) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let lv = tc.get_level(duel);
    let sg = duel.select_matching_card(tp, |c, duel| filter(c, duel, lv), tp, LOCATION_GRAVE, 0, 1, 1, Some(tc.into()))?;
    if !sg.is_empty() {
        duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, sg)?;
    }
    Ok(())
}
