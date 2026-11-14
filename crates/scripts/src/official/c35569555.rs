// Dogmatikamatrix
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 35569555;

pub const LISTED_SERIES: &[u16] = &[SET_DOGMATIKA];
// Dogmatikalamity and White Knight of Dogmatika in the Deck to add, and White Knight on the field for the second effect.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (31002402, DECK),
    (31002402, DECK),
    (40352445, DECK),
    (60303688, DECK),
    (40352445, MZONE),
    (40352445, MZONE),
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
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, gycon);
    e2.set_target(duel, gytg);
    e2.set_operation(duel, gyop);
    c.register_effect(duel, e2);
}

fn thfilter1(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DOGMATIKA]) && (c.is_ritual_monster(duel) || c.is_ritual_spell(duel)) && c.is_able_to_hand(duel)
}

fn thfilter2(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DOGMATIKA]) && c.is_able_to_hand(duel)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(thfilter1, tp, LOCATION_DECK, 0, None);
    if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 0))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let sg = g.select(duel, tp, 1, 1, None)?;
        duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, sg)?;
        let g = duel.get_matching_group(thfilter2, tp, LOCATION_DECK, 0, None);
        if !g.is_empty() && duel.get_field_group_count(tp, 0, LOCATION_MZONE) > 0 && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
            let sg = g.select(duel, tp, 1, 1, None)?;
            duel.break_effect()?;
            duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, sg)?;
        }
    }
    Ok(())
}

fn gyfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_DOGMATIKA]) && c.is_type(duel, TYPE_RITUAL)
}

fn gycon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(gyfilter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn gytg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.get_field_group_count(a.tp, LOCATION_EXTRA, LOCATION_EXTRA) > 0);
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, PLAYER_EITHER, LOCATION_EXTRA as i32);
    Ok(false)
}

fn gyop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g1 = duel.get_field_group(tp, LOCATION_EXTRA, 0);
    let g2 = duel.get_field_group(tp, 0, LOCATION_EXTRA);
    if g1.is_empty() && g2.is_empty() {
        return Ok(());
    }
    let op = duel.select_effect(
        tp,
        &[(!g1.is_empty(), aux::stringid(ID, 3)), (!g2.is_empty(), aux::stringid(ID, 4))],
    )?;
    let g = if op == Some(1) { g1 } else { g2 };
    if op == Some(2) {
        duel.confirm_cards(tp, g)?;
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let tg = g.filter_select(duel, tp, |c, duel| c.is_monster(duel) && c.is_able_to_grave(duel), 1, 1, None)?;
    if !tg.is_empty() {
        duel.break_effect()?;
        duel.send_to_grave(tg, REASON_EFFECT)?;
        if op == Some(2) {
            duel.shuffle_extra(1 - tp)?;
        }
    }
    Ok(())
}
