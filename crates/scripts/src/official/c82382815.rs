// Champion's Vigilance
use crate::prelude::*;
use crate::testing::{Check, MZONE, SZONE};

const ID: u32 = 82382815;

// Set, with a Level 8 Normal Monster (Blue-Eyes White Dragon) to enable it against the opponent's summons and activations.
pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE), (ID, SZONE), (CARD_BLUEEYES_W_DRAGON, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DISABLE_SUMMON + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_SUMMON);
    e1.set_condition(duel, condition1);
    e1.set_target(duel, target1);
    e1.set_operation(duel, activate1);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_FLIP_SUMMON);
    c.register_effect(duel, e2);
    let e3 = e1.clone_effect(duel);
    e3.set_code(duel, EVENT_SPSUMMON);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e4.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e4.set_code(duel, EVENT_CHAINING);
    e4.set_condition(duel, condition2);
    e4.set_target(duel, stock::negate_destroy_target);
    e4.set_operation(duel, activate2);
    c.register_effect(duel, e4);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_level_above(duel, 7) && c.is_type(duel, TYPE_NORMAL)
}

fn condition1(duel: &Duel, a: &Args) -> bool {
    duel.get_current_chain_full(true) == 0 && duel.is_existing_matching_card(cfilter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn target1(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let count = a.eg.map_or(0, |eg| eg.len()) as u8;
    duel.set_operation_info(0, CATEGORY_DISABLE_SUMMON, a.eg, count, 0, 0);
    duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, count, 0, 0);
    Ok(false)
}

fn activate1(duel: &mut Duel, a: &Args) -> Script<()> {
    let eg = a.eg.unwrap_or_default();
    duel.negate_summon(eg);
    duel.destroy(eg, REASON_EFFECT)?;
    Ok(())
}

fn condition2(duel: &Duel, a: &Args) -> bool {
    a.re.is_some_and(|re| re.is_has_type(duel, EFFECT_TYPE_ACTIVATE))
        && duel.is_chain_negatable(a.ev as u8)
        && duel.is_existing_matching_card(cfilter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn activate2(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.negate_activation(a.ev as u8)
        && let Some(re) = a.re
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}
