// Infernoble Knight - Roland
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 81570454;

pub const LISTED_NAMES: &[u32] = &[ID];
pub const LISTED_SERIES: &[u16] = &[SET_NOBLE_KNIGHT];

// In hand to equip to Elemental HERO Burstinatrix, and in the Deck for Infernoble Knight Ogier to send, with Infernoble Arms -
// Durendal there to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, DECK),
    (ID, DECK),
    (21351206, HAND),
    (21351206, HAND),
    (58932615, MZONE),
    (58932615, MZONE),
    (37478723, DECK),
    (37478723, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_category(duel, CATEGORY_EQUIP);
    e1.set_target(duel, equiptg);
    e1.set_operation(duel, equipop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_operation(duel, regop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_PHASE + PHASE_END);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).get_flag_effect(duel, ID) > 0);
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
}

fn equiptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(
            duel.get_location_count(tp, LOCATION_SZONE) > 0 && duel.is_existing_target(CardId::is_faceup, tp, LOCATION_MZONE, 0, 1, None)
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    duel.select_target(tp, CardId::is_faceup, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_EQUIP, Some(a.e.get_handler(duel).into()), 1, 0, 0);
    Ok(false)
}

fn equipop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || (c.is_location(duel, LOCATION_MZONE) && c.is_facedown(duel)) {
        return Ok(());
    }
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if duel.get_location_count(tp, LOCATION_SZONE) <= 0 || tc.is_facedown(duel) || !tc.is_relate_to_effect(duel, e) {
        duel.send_to_grave(c, REASON_EFFECT)?;
        return Ok(());
    }
    duel.equip_full(tp, c, tc, true, false)?;
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_EQUIP_LIMIT);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    e1.set_value(duel, Value::Card(super::c38423248::equip_limit));
    e1.set_label_object_group(duel, Some(tc.into()));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_EQUIP);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_value(duel, 500);
    e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    c.register_effect(duel, e2);
    Ok(())
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    a.e.get_handler(duel)
        .register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    ((c.is_race(duel, RACE_WARRIOR) && c.is_attribute(duel, ATTRIBUTE_FIRE)) || c.is_type(duel, TYPE_EQUIP))
        && c.is_able_to_hand(duel)
        && !c.is_code(duel, ID)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
