// The True Sun God
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 11587414;
const IMMORTAL_PHOENIX: u32 = 10000090;

pub const LISTED_NAMES: &[u32] = &[CARD_RA, ID, IMMORTAL_PHOENIX];
// In the hand to activate, with The Winged Dragon of Ra in the Deck to search and on the field, and Immortal Phoenix in the Deck to send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (CARD_RA, DECK),
    (CARD_RA, MZONE),
    (IMMORTAL_PHOENIX, DECK),
    (IMMORTAL_PHOENIX, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_ATTACK);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e2.set_target_card(duel, |duel, _, c| {
        !c.is_code(duel, CARD_RA) && c.is_status(duel, STATUS_SPSUMMON_TURN)
    });
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOGRAVE);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_target(duel, tgtg);
    e3.set_operation(duel, tgop);
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_code(duel, CARD_RA) || c.lists_code(duel, CARD_RA)) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
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

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, IMMORTAL_PHOENIX) && c.is_able_to_grave(duel)
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_RA) && c.is_able_to_grave(duel) && c.is_faceup(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok((a.e.get_handler(duel).is_able_to_grave(duel)
            || duel.is_existing_matching_card(cfilter, tp, LOCATION_DECK, 0, 1, None))
            && duel.is_existing_matching_card(tgfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 2, tp, (LOCATION_DECK | LOCATION_ONFIELD) as i32);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let mut tg = duel.get_matching_group(cfilter, tp, LOCATION_DECK, 0, None);
    if c.is_relate_to_effect(duel, e) && c.is_able_to_grave(duel) {
        tg.insert(c);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let tc = tg.select(duel, tp, 1, 1, None)?.get_first();
    if let Some(tc) = tc
        && duel.send_to_grave(tc, REASON_EFFECT)? > 0
        && tc.is_location(duel, LOCATION_GRAVE)
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
        let sg = duel.select_matching_card(tp, tgfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
        if !sg.is_empty() {
            duel.break_effect()?;
            duel.send_to_grave(sg, REASON_EFFECT)?;
        }
    }
    Ok(())
}
