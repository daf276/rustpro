// Smashing Ground
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 97169186;

// In the hand, with two equal monsters of the opponent's for the tie: Sangan.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (26202165, MZONE | OPPONENT), (26202165, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_defense_above(duel, 0)
}

fn max_defense(duel: &Duel, tp: u8) -> Option<CardSet> {
    let g = duel.get_matching_group(filter, tp, 0, LOCATION_MZONE, None);
    g.get_max_group(duel, |c, duel| c.get_defense(duel).into()).map(|(tg, _)| tg)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, 0, LOCATION_MZONE, 1, None));
    }
    let tg = max_defense(duel, tp);
    duel.set_operation_info(0, CATEGORY_DESTROY, tg, 1, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(tg) = max_defense(duel, tp) else { return Ok(()) };
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
