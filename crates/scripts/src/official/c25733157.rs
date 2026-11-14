// Sky Striker Mecha - Eagle Booster
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 25733157;

// The opponent's monsters to target, three Spells in the Graveyard for the second effect.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (ID, GRAVE),
    (ID, GRAVE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    e1.set_hint_timing(duel, 0, TIMING_BATTLE_START | TIMING_END_PHASE);
    c.register_effect(duel, e1);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.get_sequence(duel) < 5
}

pub fn condition(duel: &Duel, a: &Args) -> bool {
    !duel.is_existing_matching_card(cfilter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_faceup(duel) && tc.is_relate_to_effect(duel, e) {
        let c = e.get_handler(duel);
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3100);
        e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
        e1.set_value(duel, Value::Effect(|duel, e, re| e.get_handler(duel) != re.get_owner(duel)));
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
        if duel.get_matching_group_count(CardId::is_spell, tp, LOCATION_GRAVE, 0, None) >= 3 {
            let e2 = duel.create_effect(c);
            e2.set_description(duel, 3000);
            e2.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
            e2.set_type(duel, EFFECT_TYPE_SINGLE);
            e2.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
            e2.set_value(duel, 1);
            e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
            tc.register_effect(duel, e2);
        }
    }
    Ok(())
}
