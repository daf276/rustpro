// Legacy of Yata-Garasu
use crate::prelude::*;

const ID: u32 = 30461781;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, stock::draw);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_SPIRIT) && c.is_faceup(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 1));
    }
    let mut d = 1;
    if duel.get_field_group_count(tp, LOCATION_DECK, 0) > 1
        && duel.is_existing_matching_card(filter, tp, 0, LOCATION_MZONE, 1, None)
        && duel.select_option(tp, &[aux::stringid(ID, 0), aux::stringid(ID, 1)])? == 1
    {
        d = 2;
    }
    duel.set_target_player(tp);
    duel.set_target_param(d);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, d as i32);
    Ok(false)
}
