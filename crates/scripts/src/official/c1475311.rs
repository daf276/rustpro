// Allure of Darkness
use crate::prelude::*;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DRAW + CATEGORY_REMOVE + CATEGORY_HANDES);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_affected_by_effect(tp, 30459350).is_none() && duel.is_player_can_draw(tp, 2));
    }
    duel.set_target_player(tp);
    duel.set_target_param(2);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 2);
    duel.set_possible_operation_info(0, CATEGORY_HANDES, None, 2, tp, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.draw(p, d, REASON_EFFECT)?;
    duel.shuffle_hand(p)?;
    duel.break_effect()?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(p, |c, duel| c.is_attribute(duel, ATTRIBUTE_DARK), p, LOCATION_HAND, 0, 1, 1, None)?;
    if g.is_empty() {
        let sg = duel.get_field_group(p, LOCATION_HAND, 0);
        duel.send_to_grave(sg, REASON_EFFECT)?;
    } else if duel.remove(g, POS_FACEUP, REASON_EFFECT)? == 0 {
        duel.confirm_cards(1 - p, g)?;
        duel.shuffle_hand(p)?;
    }
    Ok(())
}
