// Black Pendant
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 65169794;

struct Procedure;

impl EquipProcedure for Procedure {}

// In the hand to equip a monster (Shapesnatch) and destroyed with it by the opponent's Dark Hole.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (4035199, MZONE),
    (4035199, MZONE),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Equip::add_procedure::<Procedure>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_EQUIP);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_value(duel, 500);
    c.register_effect(duel, e2);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 0));
    e4.set_category(duel, CATEGORY_DAMAGE);
    e4.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e4.set_code(duel, EVENT_TO_GRAVE);
    e4.set_condition(duel, |duel, a| a.e.get_handler(duel).is_previous_location(duel, LOCATION_ONFIELD));
    e4.set_target(duel, damtg);
    e4.set_operation(duel, damop);
    c.register_effect(duel, e4);
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(1 - tp);
    duel.set_target_param(500);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 500);
    Ok(false)
}

fn damop(duel: &mut Duel, _: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.damage(p, d, REASON_EFFECT)?;
    Ok(())
}
