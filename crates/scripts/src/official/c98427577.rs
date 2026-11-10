// Scrap-Iron Scarecrow
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT, SZONE};

const ID: u32 = 98427577;

// Set, against the opponent's Terra the Terribles attacking.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (63308047, MZONE | OPPONENT),
    (63308047, MZONE | OPPONENT),
    (63308047, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SET);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e1.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tg = duel.get_attacker();
    if let Some(chkc) = chkc {
        return Ok(Some(chkc) == tg);
    }
    let Some(tg) = tg else { return Ok(false) };
    if chk == 0 {
        return Ok(tg.is_on_field(duel) && tg.is_can_be_effect_target(duel, a.e));
    }
    duel.set_target_card(tg);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.negate_attack()?;
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    if c.is_ssetable_full(duel, true, None) && e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        duel.break_effect()?;
        c.cancel_to_grave(duel);
        duel.change_position(c, POS_FACEDOWN)?;
        duel.raise_event(c, EVENT_SSET, Some(e), REASON_EFFECT, tp, tp, 0)?;
    }
    Ok(())
}
