// Black Metal Dragon
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};
use engine::constants::{MSG_CONFIRM_CARDS, MSG_EQUIP};

const ID: u32 = 93969023;

// In the hand and on the field with Red-Eyes Black Dragon to equip to, which is also in the Deck to search; Dark Hole from the other side.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (74677422, MZONE),
    (74677422, DECK),
    (74677422, DECK),
    (53129443, HAND | OPPONENT),
];
// Both effects have no description, so the equip and the search are checked by what they do.
pub const CHECKS: &[Check] = &[Check::Message(MSG_EQUIP), Check::Message(MSG_CONFIRM_CARDS)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_EQUIP);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_HAND | LOCATION_MZONE);
    e1.set_target(duel, eqtg);
    e1.set_operation(duel, eqop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_previous_location(duel, LOCATION_ONFIELD));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_RED_EYES])
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && filter(chkc, duel));
    }
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(
            duel.get_location_count(tp, LOCATION_SZONE) > 0 && duel.is_existing_target(filter, tp, LOCATION_MZONE, 0, 1, Some(c.into()))
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    duel.select_target(tp, filter, tp, LOCATION_MZONE, 0, 1, 1, Some(c.into()))?;
    duel.set_operation_info(0, CATEGORY_EQUIP, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || c.is_location(duel, LOCATION_MZONE) && c.is_facedown(duel) {
        return Ok(());
    }
    let Some(tc) = duel.get_first_target() else {
        return Ok(());
    };
    if duel.get_location_count(tp, LOCATION_SZONE) <= 0
        || tc.get_controler(duel) != tp
        || tc.is_facedown(duel)
        || !tc.is_relate_to_effect(duel, e)
    {
        duel.send_to_grave(c, REASON_EFFECT)?;
        return Ok(());
    }
    duel.equip_full(tp, c, tc, true, false)?;
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_EQUIP_LIMIT);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    e1.set_label_object_group(duel, Some(tc.into()));
    e1.set_value(duel, Value::Card(super::c38423248::equip_limit));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_EQUIP);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_value(duel, 600);
    e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    c.register_effect(duel, e2);
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RED_EYES]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
