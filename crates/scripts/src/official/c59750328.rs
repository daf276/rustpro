// Card of Demise
use crate::prelude::*;
use crate::testing::HAND;

const ID: u32 = 59750328;

// In the hand.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_activity_count(tp, ACTIVITY_SPSUMMON) == 0);
    }
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    e1.set_target_range(duel, 1, 0);
    duel.register_effect(e1, tp);
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let handler = a.e.get_handler(duel);
    let ct = 3 - duel.get_matching_group_count(aux::TRUE, tp, LOCATION_HAND, 0, Some(handler.into())) as i32;
    if chk == 0 {
        return Ok(ct > 0 && duel.is_player_can_draw(tp, ct as usize));
    }
    duel.set_target_player(tp);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, ct);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let ct = 3 - duel.get_field_group_count(p, LOCATION_HAND, 0) as i32;
    if ct > 0 {
        duel.draw(p, ct as u32, REASON_EFFECT)?;
    }
    let c = e.get_handler(duel);
    if e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_code(duel, EFFECT_CHANGE_DAMAGE);
        e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
        e1.set_target_range(duel, 0, 1);
        e1.set_value(duel, 0);
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e1, p);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_NO_EFFECT_DAMAGE);
        e2.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e2, p);
    }
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EVENT_PHASE + PHASE_END);
    e3.set_count_limit(duel, 1, 0);
    e3.set_reset(duel, RESET_PHASE | PHASE_END);
    e3.set_operation(duel, tgop);
    duel.register_effect(e3, p);
    Ok(())
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_field_group(a.e.get_owner_player(duel), LOCATION_HAND, 0);
    duel.send_to_grave(g, REASON_EFFECT)?;
    Ok(())
}
