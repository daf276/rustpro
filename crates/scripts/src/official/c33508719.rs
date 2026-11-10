// Morphing Jar
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 33508719;

// Face-down on the field to be flipped by an attack from an opponent's Sangan.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (26202165, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_HANDES + CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_FLIP);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, _: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, PLAYER_ALL, 0);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, PLAYER_ALL, 5);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_field_group(tp, LOCATION_HAND, LOCATION_HAND);
    if !g.is_empty() {
        duel.send_to_grave(g, REASON_DISCARD | REASON_EFFECT)?;
    }
    duel.break_effect()?;
    duel.draw(tp, 5, REASON_EFFECT)?;
    duel.draw(1 - tp, 5, REASON_EFFECT)?;
    Ok(())
}
