// Destructive Daruma Karma Cannon
use crate::prelude::*;

const ID: u32 = 30748475;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_POSITION + CATEGORY_TOGRAVE + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_can_turn_set, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    let g = duel.get_matching_group(CardId::is_can_turn_set, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    duel.set_operation_info(0, CATEGORY_POSITION, Some(g), g.len() as u8, tp, POS_FACEDOWN_DEFENSE as i32);
    let tg = duel.get_matching_group(|c, duel| !c.is_can_turn_set(duel), tp, LOCATION_MZONE, LOCATION_MZONE, None);
    if !tg.is_empty() {
        duel.set_operation_info(0, CATEGORY_TOGRAVE, Some(tg), tg.len() as u8, 0, 0);
    }
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(CardId::is_can_turn_set, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    if g.is_empty() || duel.change_position(g, POS_FACEDOWN_DEFENSE)? <= 0 {
        return Ok(());
    }
    let turn_p = duel.get_turn_player();
    let g1 = duel.get_matching_group(CardId::is_faceup, turn_p, LOCATION_MZONE, 0, None);
    let g2 = duel.get_matching_group(CardId::is_faceup, turn_p, 0, LOCATION_MZONE, None);
    if g1.is_empty() && g2.is_empty() {
        return Ok(());
    }
    duel.break_effect()?;
    if !g1.is_empty() {
        duel.send_to_grave_full(g1, REASON_RULE, PLAYER_NONE, turn_p)?;
    }
    if !g2.is_empty() {
        duel.send_to_grave_full(g2, REASON_RULE, PLAYER_NONE, 1 - turn_p)?;
    }
    Ok(())
}
