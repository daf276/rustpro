// Hyper Hammerhead
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 2671330;

// Two on the field to attack, and the opponent's monsters to attack into.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (13597785, MZONE | OPPONENT), (13597785, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_DAMAGE_STEP_END);
    e1.set_condition_mut(duel, retcon);
    e1.set_target(duel, rettg);
    e1.set_operation(duel, retop);
    c.register_effect(duel, e1);
}

fn retcon(duel: &mut Duel, a: &Args) -> bool {
    let t = a.e.get_handler(duel).get_battle_target(duel);
    a.e.set_label_object_group(duel, t.map(CardSet::from));
    t.is_some_and(|t| t.is_relate_to_battle(duel))
}

fn rettg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let g = a.e.get_label_object_group(duel);
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, 0, 0);
    Ok(false)
}

fn retop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(t) = a.e.get_label_object_group(duel).and_then(|g| g.get_first())
        && t.is_relate_to_battle(duel)
    {
        duel.send_to_hand(t, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
