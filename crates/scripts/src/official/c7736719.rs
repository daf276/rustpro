// Vortex Trooper
use crate::prelude::*;
use crate::testing::{Check, HAND, OPPONENT};

const ID: u32 = 7736719;

// In the hand to Normal Summon, destroyed by the opponent's Raigeki and Dark Hole.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (12580477, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TODECK + CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_target(duel, drtg);
    e2.set_operation(duel, stock::draw);
    c.register_effect(duel, e2);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 2) && duel.is_existing_matching_card(CardId::is_able_to_deck, tp, LOCATION_HAND, 0, 2, None));
    }
    duel.set_operation_info(0, CATEGORY_TODECK, None, 2, tp, LOCATION_HAND as i32);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 2);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(CardId::is_able_to_deck, tp, LOCATION_HAND, 0, None);
    if g.len() < 2 || !duel.is_player_can_draw(tp, 0) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let dg = g.select(duel, tp, 2, 2, None)?;
    duel.send_to_deck(dg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    duel.shuffle_deck(tp)?;
    duel.break_effect()?;
    duel.draw(tp, 2, REASON_EFFECT)?;
    Ok(())
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(tp);
    duel.set_target_param(1);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    Ok(false)
}
