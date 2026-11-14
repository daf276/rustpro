// Gravekeeper's Inscription
use crate::prelude::*;
use crate::testing::{Check, HAND};

const ID: u32 = 59494222;

// In the hand to activate at the start of Main Phase 1.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_condition(duel, |duel, _| duel.is_phase(PHASE_MAIN1) && !duel.check_phase_activity());
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn flagcheck(duel: &Duel, tp: u8) -> [bool; 3] {
    [1, 2, 3].map(|n| duel.get_flag_effect(tp, ID + n) == 0)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && flagcheck(duel, a.tp).contains(&true))
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let [b1, b2, b3] = flagcheck(duel, tp);
    let options = [(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2)), (b3, aux::stringid(ID, 3))];
    let Some(op) = duel.select_effect(tp, &options)? else {
        return Ok(());
    };
    let code = match op {
        1 => EFFECT_CANNOT_ACTIVATE,
        2 => EFFECT_CANNOT_REMOVE,
        _ => EFFECT_CANNOT_SPECIAL_SUMMON,
    };
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, op as u64 + 3));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, code);
    e1.set_target_range(duel, 1, 1);
    if op == 1 {
        e1.set_value(
            duel,
            Value::EffectPlayer(|duel, _, re, _| re.get_activate_location(duel) == LOCATION_GRAVE),
        );
    } else {
        e1.set_target_card(duel, |duel, _, c| c.is_location(duel, LOCATION_GRAVE));
    }
    e1.set_reset(duel, RESET_PHASE | PHASE_END | RESET_OPPO_TURN);
    duel.register_effect(e1, tp);
    duel.register_flag_effect(tp, ID + op as u32, RESET_PHASE | PHASE_END, 0, 1);
    Ok(())
}
