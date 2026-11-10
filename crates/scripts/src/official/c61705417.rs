// Graverobber
use crate::prelude::*;
use crate::testing::{GRAVE, HAND, OPPONENT};

const ID: u32 = 61705417;

const RESET_LEAVE: u32 = RESET_EVENT | RESET_TOGRAVE | RESET_REMOVE | RESET_TEMP_REMOVE | RESET_TODECK | RESET_PHASE | PHASE_END;

// In the hand, with Dark Holes and Pot of Greed in the opponent's GY to take.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (53129443, GRAVE | OPPONENT),
    (53129443, GRAVE | OPPONENT),
    (55144522, GRAVE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_spell(duel) && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_GRAVE) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, 0, LOCATION_GRAVE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, filter, tp, 0, LOCATION_GRAVE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    duel.send_to_hand(tc, tp, REASON_EFFECT)?;
    duel.confirm_cards(1 - tp, tc)?;
    tc.register_flag_effect(duel, ID, RESET_LEAVE, 0, 1);
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_code(duel, EVENT_PHASE | PHASE_END);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, tgcon);
    e1.set_operation(duel, tgop);
    e1.set_label_object_group(duel, Some(tc.into()));
    e1.set_reset(duel, RESET_LEAVE);
    duel.register_effect(e1, tp);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EVENT_CHAINING);
    e3.set_condition(duel, actcon);
    e3.set_operation(duel, actop);
    e3.set_label_object_group(duel, Some(tc.into()));
    e3.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e3, tp);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e4.set_code(duel, EVENT_CHAIN_SOLVED);
    e4.set_condition(duel, damcon);
    e4.set_operation(duel, damop);
    e4.set_label_object_group(duel, Some(tc.into()));
    e4.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e4, tp);
    Ok(())
}

fn label_card(duel: &Duel, e: EffectId) -> Option<CardId> {
    e.get_label_object_group(duel).and_then(|g| g.get_first())
}

fn tgcon(duel: &Duel, a: &Args) -> bool {
    label_card(duel, a.e).is_some_and(|tc| tc.get_controler(duel) != tc.get_owner(duel) && tc.get_flag_effect(duel, ID) != 0)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = label_card(duel, a.e) {
        duel.send_to_grave(tc, REASON_EFFECT)?;
    }
    Ok(())
}

fn flagged_handler(duel: &Duel, a: &Args, flag: u32) -> Option<CardId> {
    let rc = a.re?.get_handler(duel);
    (a.rp == a.tp && Some(rc) == label_card(duel, a.e) && rc.get_flag_effect(duel, flag) != 0).then_some(rc)
}

fn actcon(duel: &Duel, a: &Args) -> bool {
    flagged_handler(duel, a, ID).is_some()
}

fn actop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(rc) = a.re.map(|re| re.get_handler(duel)) {
        rc.register_flag_effect(duel, ID + 1, RESET_CHAIN, 0, 1);
    }
    a.e.reset(duel);
    Ok(())
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    flagged_handler(duel, a, ID + 1).is_some()
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(rc) = a.re.map(|re| re.get_handler(duel)) {
        rc.reset_flag_effect(duel, ID + 1);
    }
    duel.damage(a.tp, 2000, REASON_EFFECT)?;
    a.e.reset(duel);
    Ok(())
}
