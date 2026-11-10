// Dark Bribe
use crate::prelude::*;
use crate::testing::{HAND, OPPONENT};

const ID: u32 = 77538567;

// Raigeki in the opponent's hand to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (12580477, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY + CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    a.re.is_some_and(|re| re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)) && a.rp != a.tp && duel.is_chain_negatable(a.ev as u8)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_draw(1 - tp, 1));
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    if let Some(re) = a.re {
        let rc = re.get_handler(duel);
        if rc.is_destructable(duel) && rc.is_relate_to_effect(duel, re) {
            duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, 0, 0);
        }
    }
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, 1 - tp, 1);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (tp, re) = (a.tp, a.re);
    if duel.negate_activation(a.ev as u8) && re.is_some_and(|re| re.get_handler(duel).is_relate_to_effect(duel, re)) {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    duel.draw(1 - tp, 1, REASON_EFFECT)?;
    Ok(())
}
