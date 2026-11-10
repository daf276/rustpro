// Paleozoic Canadia
use super::c64765016::paleozoic_spsummon;
use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE, OPPONENT, SZONE};
use engine::constants::MSG_POS_CHANGE;

const ID: u32 = 98414735;

// A copy in the Graveyard to Special Summon in response to another Set copy activating, and an opponent's monster to turn
// face-down.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, GRAVE),
    (ID, SZONE),
    (ID, SZONE),
    (30655537, MZONE | OPPONENT),
    (30655537, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::SpecialSummoned, Check::Message(MSG_POS_CHANGE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_POSITION + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_hint_timing(duel, TIMING_BATTLE_PHASE, TIMINGS_CHECK_MONSTER | TIMING_BATTLE_PHASE);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = paleozoic_spsummon::<ID>(duel, c);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_can_turn_set(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    let g = duel.select_target(tp, filter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_POSITION, g, 1, tp, POS_FACEDOWN_DEFENSE as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && tc.is_faceup(duel)
    {
        duel.change_position(tc, POS_FACEDOWN_DEFENSE)?;
    }
    Ok(())
}
