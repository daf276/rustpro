// Card of Sanctity
use crate::prelude::*;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DRAW);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let location = LOCATION_HAND | LOCATION_ONFIELD;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_remove_as_cost, tp, location, 0, 1, Some(c.into())));
    }
    let g = duel.get_matching_group(CardId::is_able_to_remove_as_cost, tp, location, 0, Some(c.into()));
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 0));
    }
    let ht = duel.get_field_group_count(tp, LOCATION_HAND, 0) as i32;
    duel.set_target_player(tp);
    duel.set_target_param((2 - ht) as u32);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 2 - ht);
    Ok(false)
}

fn operation(duel: &mut Duel, _: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let ht = duel.get_field_group_count(p, LOCATION_HAND, 0);
    if ht < 2 {
        duel.draw(p, 2 - ht as u32, REASON_EFFECT)?;
    }
    Ok(())
}
