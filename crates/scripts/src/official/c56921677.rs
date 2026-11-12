// Heraldic Beast Basilisk
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 56921677;

// On the field, with the opponent's monsters to battle.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (30655537, MZONE | OPPONENT), (30655537, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_BATTLED);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).get_battle_target(duel).is_some());
    e1.set_target(duel, target);
    e1.set_operation(duel, stock::destroy_first_target);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tc = a.e.get_handler(duel).get_battle_target(duel);
    duel.set_target_card(tc.map(CardSet::from).unwrap_or_default());
    duel.set_operation_info(0, CATEGORY_DESTROY, tc.map(CardSet::from), 1, 0, 0);
    Ok(false)
}
