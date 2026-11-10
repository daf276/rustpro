// Judgment of Anubis
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 55256016;

// Set, with copies in the hand to discard, and Raigeki in the opponent's hand to negate, and a monster of the opponent's to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (ID, HAND),
    (12580477, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_condition(duel, condition);
    e1.set_cost(duel, super::c77414722::cost);
    e1.set_target(duel, super::c77414722::target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_on_field(duel) && c.is_spell_trap(duel)
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    if a.tp == a.ep || !duel.is_chain_negatable(a.ev as u8) {
        return false;
    }
    if !(re.is_spell_effect(duel) && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)) {
        return false;
    }
    duel.get_operation_info(a.ev as u8, CATEGORY_DESTROY).is_some_and(|info| {
        info.op_cards
            .is_some_and(|tg| i32::from(info.op_count) + tg.filter_count(duel, cfilter, None) as i32 - tg.len() as i32 > 0)
    })
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (tp, Some(re)) = (a.tp, a.re) else { return Ok(()) };
    if duel.negate_activation(a.ev as u8) && re.get_handler(duel).is_relate_to_effect(duel, re) {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    let dg = duel.get_matching_group(CardId::is_faceup, tp, 0, LOCATION_MZONE, None);
    if !dg.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 0))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let des = dg.select(duel, tp, 1, 1, None)?;
        duel.hint_selection(des);
        let atk = des.get_first().map_or(0, |tc| tc.get_attack(duel));
        duel.break_effect()?;
        if duel.destroy(des, REASON_EFFECT)? > 0 {
            duel.damage(1 - tp, atk.max(0) as u32, REASON_EFFECT)?;
        }
    }
    Ok(())
}
