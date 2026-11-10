// One Day of Peace
use crate::prelude::*;
use crate::testing::HAND;

const ID: u32 = 33782437;

pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_category(duel, CATEGORY_DRAW);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 1) && duel.is_player_can_draw(1 - tp, 1));
    }
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, PLAYER_ALL, 1);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let d1 = duel.draw(tp, 1, REASON_EFFECT)?;
    let d2 = duel.draw(1 - tp, 1, REASON_EFFECT)?;
    if d1 == 0 || d2 == 0 {
        return Ok(());
    }
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CHANGE_DAMAGE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_target_range(duel, 1, 1);
    e1.set_value(duel, 0);
    e1.set_reset_full(duel, RESET_PHASE | PHASE_END, 2);
    duel.register_effect(e1, tp);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_NO_EFFECT_DAMAGE);
    e2.set_reset_full(duel, RESET_PHASE | PHASE_END, 2);
    duel.register_effect(e2, tp);
    let e3 = duel.create_effect(c);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_reset_full(duel, RESET_PHASE | PHASE_END, 2);
    e3.set_target_range(duel, 1, 1);
    duel.register_effect(e3, tp);
    Ok(())
}
