// Assault Slash
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 40012727;

pub const LISTED_SERIES: &[u16] = &[SET_ASSAULT_MODE];
// In the hand with an Assault Mode monster (Crimson Blader/Assault Mode) on the field, and an opponent's monster to destroy.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (50604072, MZONE), (26202165, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_ASSAULT_MODE])
}

fn condition(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(cfilter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    let dg = duel.get_matching_group(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(dg), dg.len() as u8, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let dg = duel.get_matching_group(CardId::is_faceup, a.tp, LOCATION_MZONE, LOCATION_MZONE, None);
    duel.destroy(dg, REASON_EFFECT)?;
    Ok(())
}
