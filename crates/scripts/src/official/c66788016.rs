// Fissure
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 66788016;

// In hand against opponent monsters, some of equal ATK (Swordsman of Landstar, Shapesnatch).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (3573512, MZONE | OPPONENT),
    (3573512, MZONE | OPPONENT),
    (4035199, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Resolves];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_faceup, tp, 0, LOCATION_MZONE, 1, None));
    }
    let g = duel.get_matching_group(CardId::is_faceup, tp, 0, LOCATION_MZONE, None);
    let tg = g.get_min_group(duel, |c, duel| c.get_attack(duel).into()).map(|(tg, _)| tg);
    duel.set_operation_info(0, CATEGORY_DESTROY, tg, 1, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(CardId::is_faceup, tp, 0, LOCATION_MZONE, None);
    let Some((tg, _)) = g.get_min_group(duel, |c, duel| c.get_attack(duel).into()) else {
        return Ok(());
    };
    if tg.len() > 1 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let sg = tg.select(duel, tp, 1, 1, None)?;
        duel.hint_selection(sg);
        duel.destroy(sg, REASON_EFFECT)?;
    } else {
        duel.destroy(tg, REASON_EFFECT)?;
    }
    Ok(())
}
