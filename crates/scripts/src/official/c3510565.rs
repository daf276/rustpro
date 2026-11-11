// Stealth Bird
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 3510565;

// Copies on the field and in the hand, which the player Sets and Flip Summons.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (ID, MZONE), (ID, HAND), (ID, HAND)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_POSITION + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DAMAGE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EVENT_FLIP_SUMMON_SUCCESS);
    e2.set_target(duel, damtg);
    e2.set_operation(duel, stock::damage);
    c.register_effect(duel, e2);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_can_turn_set(duel) && c.get_flag_effect(duel, ID) == 0);
    }
    c.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END & !RESET_TURN_SET, 0, 1);
    duel.set_operation_info(0, CATEGORY_POSITION, Some(c.into()), 1, a.tp, POS_FACEDOWN_DEFENSE as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && c.is_faceup(duel) {
        duel.change_position(c, POS_FACEDOWN_DEFENSE)?;
    }
    Ok(())
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(1 - a.tp);
    duel.set_target_param(1000);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - a.tp, 1000);
    Ok(false)
}
