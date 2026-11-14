// Double Cyclone
use crate::prelude::*;
use crate::testing::{HAND, OPPONENT, SZONE};

const ID: u32 = 75652080;

// Spells and Traps on both sides to destroy.
pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE), (ID, SZONE), (ID, HAND), (ID, SZONE | OPPONENT), (ID, SZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_END_PHASE);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    if chkc.is_some() {
        return Ok(false);
    }
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, 0, LOCATION_ONFIELD, 1, None)
            && duel.is_existing_target(filter, tp, LOCATION_ONFIELD, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g1 = duel.select_target(tp, filter, tp, LOCATION_ONFIELD, 0, 1, 1, Some(c.into()))?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g2 = duel.select_target(tp, filter, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    let g = g1.unwrap_or_default().union(g2.unwrap_or_default());
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 2, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel
        .get_chain_info::<Option<CardSet>>(0, CHAININFO_TARGET_CARDS)
        .unwrap_or_default();
    let sg = g.filter(duel, |c, duel| c.is_relate_to_effect(duel, a.e), None);
    if !sg.is_empty() {
        duel.destroy(sg, REASON_EFFECT)?;
    }
    Ok(())
}
