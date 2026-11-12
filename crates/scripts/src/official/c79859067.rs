// R.B. Next Phase
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 79859067;

// Set with R.B. Lambda Cannon on the field, to negate the opponent's Effect Monster (Effect Veiler, from the hand).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (81794107, MZONE),
    (97268402, HAND | OPPONENT),
    (97268402, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY + CATEGORY_RECOVER);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    a.rp == 1 - a.tp
        && a.re.is_some_and(|re| re.is_monster_effect(duel))
        && duel.is_chain_negatable(a.ev as u8)
        && duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_RB])),
            a.tp,
            LOCATION_MZONE,
            0,
            1,
            None,
        )
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_field_group(tp, LOCATION_MZONE, 0);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, tp, 0);
    let Some(re) = a.re else { return Ok(false) };
    let rc = re.get_handler(duel);
    if rc.is_destructable(duel) && rc.is_relate_to_effect(duel, re) {
        duel.set_operation_info(0, CATEGORY_DESTROY, Some(g.union(a.eg.unwrap_or_default())), 2, tp, 0);
        duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, tp, 2000);
    } else {
        duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, tp, 0);
    }
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_matching_card(tp, aux::TRUE, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    if g.is_empty() {
        return Ok(());
    }
    duel.hint_selection(g);
    if duel.destroy(g, REASON_EFFECT)? > 0
        && duel.negate_activation(a.ev as u8)
        && a.re.is_some_and(|re| re.get_handler(duel).is_relate_to_effect(duel, re))
        && duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)? > 0
    {
        duel.break_effect()?;
        duel.recover(tp, 2000, REASON_EFFECT)?;
    }
    Ok(())
}
