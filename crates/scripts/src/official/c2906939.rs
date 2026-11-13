// Ashtrashen - Gateway to the Worlds Beyond
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, SZONE};

const ID: u32 = 2906939;

// Activated from the hand with face-down Ashtra Divine Domain (Trap) to send and Yoa on the field, with Yoa, Mikumari and Akasuna
// in the Deck to add (different names) and Yoa's Set from the Deck to return a card.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (1710647, MZONE),
    (1710647, HAND),
    (37839434, SZONE),
    (37839434, SZONE),
    (37839434, DECK),
    (1710647, DECK),
    (38114652, DECK),
    (38331244, DECK),
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
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_FZONE);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2a = duel.create_effect(c);
    e2a.set_description(duel, aux::stringid(ID, 1));
    e2a.set_category(duel, CATEGORY_TOHAND);
    e2a.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2a.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2a.set_code(duel, EVENT_MSET);
    e2a.set_range(duel, LOCATION_FZONE);
    e2a.set_count_limit(duel, 1, (ID, 1));
    e2a.set_condition(duel, rthcon);
    e2a.set_target(duel, rthtg);
    e2a.set_operation(duel, rthop);
    c.register_effect(duel, e2a);
    let e2b = e2a.clone_effect(duel);
    e2b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2b.set_condition(duel, |duel, a| spsuccesscon(duel, a) && rthcon(duel, a));
    c.register_effect(duel, e2b);
    let e2c = e2a.clone_effect(duel);
    e2c.set_code(duel, EVENT_SSET);
    c.register_effect(duel, e2c);
    let e2d = e2a.clone_effect(duel);
    e2d.set_code(duel, EVENT_CHANGE_POS);
    e2d.set_condition(duel, |duel, a| changeposcon(duel, a) && rthcon(duel, a));
    c.register_effect(duel, e2d);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ASHTRA]) && !c.is_field_spell(duel) && c.is_able_to_hand(duel)
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_facedown(duel) && c.is_able_to_grave(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let search_group = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
    if chk == 0 {
        return Ok(!search_group.is_empty() && duel.is_existing_target(tgfilter, tp, LOCATION_ONFIELD, 0, 1, None));
    }
    let max_target_count = search_group.get_class_count(duel, |c, duel| i64::from(c.get_code(duel)));
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_target(tp, tgfilter, tp, LOCATION_ONFIELD, 0, 1, max_target_count as u8, None)?;
    let count = g.map_or(0, |g| g.len()) as u8;
    duel.set_operation_info(0, CATEGORY_TOHAND, None, count, tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, g, count, tp, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mut tg = duel.get_target_cards(e).unwrap_or_default();
    tg.keep_matching(duel, CardId::is_facedown, None);
    let face_down_count = tg.len();
    if face_down_count > 0 {
        let g = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
        if g.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) < face_down_count {
            return Ok(());
        }
        let sg = aux::select_unselect_group(duel, g, face_down_count, face_down_count, &aux::dncheck, tp, HINTMSG_ATOHAND)?;
        if sg.len() == face_down_count && duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)? == face_down_count as i32 {
            duel.confirm_cards(1 - tp, sg)?;
            duel.shuffle_hand(tp)?;
            duel.break_effect()?;
            duel.send_to_grave(tg, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn rthcon(duel: &Duel, a: &Args) -> bool {
    let eg = a.eg.unwrap_or_default();
    let ashtra = aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_ASHTRA]));
    duel.is_existing_matching_card(ashtra, a.tp, LOCATION_MZONE, 0, 1, Some(eg))
        && duel.is_existing_matching_card(CardId::is_facedown, a.tp, LOCATION_ONFIELD, 0, 1, Some(eg))
}

fn spsuccesscon(duel: &Duel, a: &Args) -> bool {
    a.eg.unwrap_or_default().is_exists(duel, CardId::is_facedown, 1, None)
}

fn changeposconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_facedown(duel) && c.is_previous_position(duel, POS_FACEUP)
}

fn changeposcon(duel: &Duel, a: &Args) -> bool {
    a.eg.unwrap_or_default().is_exists(duel, changeposconfilter, 1, None)
}

fn rthtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_able_to_hand(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_able_to_hand, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_target(tp, CardId::is_able_to_hand, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, tp, 0);
    Ok(false)
}

fn rthop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
