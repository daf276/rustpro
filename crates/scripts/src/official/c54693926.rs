// Dark Ruler No More
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 54693926;

// Sangan, an Effect Monster, on the opponent's side to negate.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (26202165, MZONE | OPPONENT), (26202165, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_negatable_monster, tp, 0, LOCATION_MZONE, 1, None));
    }
    if e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        duel.set_chain_limit(|duel, te, _, _, _| !te.is_monster_effect(duel), 0);
    }
    duel.set_operation_info(0, CATEGORY_DISABLE, None, 1, 1 - tp, LOCATION_MZONE as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(CardId::is_negatable_monster, tp, 0, LOCATION_MZONE, None);
    let c = e.get_handler(duel);
    for tc in g {
        tc.negate_effects(duel, c, RESET_PHASE | PHASE_END, false);
    }
    if !e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CHANGE_DAMAGE);
    e1.set_target_range(duel, 0, 1);
    e1.set_value(duel, 0);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_NO_EFFECT_DAMAGE);
    duel.register_effect(e2, tp);
    Ok(())
}
