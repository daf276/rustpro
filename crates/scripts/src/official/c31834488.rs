// Book of Lunar Eclipse
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 31834488;

// Copies in the hand, with monsters on both fields to turn face-down.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (26202165, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_POSITION + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, TIMING_BATTLE_PHASE, TIMINGS_CHECK_MONSTER | TIMING_BATTLE_PHASE);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, Some(c.into())));
    }
    duel.discard_hand(tp, CardId::is_discardable, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_can_turn_set(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_MZONE, LOCATION_MZONE, 2, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    let g = duel.select_target(tp, filter, tp, LOCATION_MZONE, LOCATION_MZONE, 2, 2, None)?;
    duel.set_operation_info(0, CATEGORY_POSITION, g, 2, tp, POS_FACEDOWN_DEFENSE as i32);
    Ok(false)
}

fn chkfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_relate_to_effect(duel, e) && c.is_location(duel, LOCATION_MZONE) && c.is_faceup(duel)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let targets: Option<CardSet> = duel.get_chain_info(0, CHAININFO_TARGET_CARDS);
    let g = targets.unwrap_or_default().filter(duel, |c, duel| chkfilter(c, duel, e), None);
    if !g.is_empty() {
        duel.change_position(g, POS_FACEDOWN_DEFENSE)?;
    }
    Ok(())
}
