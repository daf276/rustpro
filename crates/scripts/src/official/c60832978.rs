// Double Resonator
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 60832978;

// In the hand and Graveyard beside Fiend Normal Monsters (White Duston, Kozaky) on the field.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (ID, GRAVE), (3557275, MZONE), (99171160, MZONE)];
pub const CHECKS: &[Check] = &[Check::Resolves];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e1.set_target(duel, target1);
    e1.set_operation(duel, operation1);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_count_limit(duel, 1, ID);
    e3.set_cost(duel, Cost::self_banish);
    e3.set_target(duel, target2);
    e3.set_operation(duel, operation2);
    c.register_effect(duel, e3);
}

fn filter1(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && !c.is_type(duel, TYPE_TUNER)
}

fn target1(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && filter1(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter1, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, filter1, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    Ok(false)
}

fn add_tuner(duel: &mut Duel, a: &Args, filter: impl Fn(CardId, &Duel) -> bool) -> Script<()> {
    let e = a.e;
    let tc = duel.get_first_target();
    unported!(duel, tc.is_none());
    if let Some(tc) = tc
        && tc.is_relate_to_effect(duel, e)
        && filter(tc, duel)
    {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_ADD_TYPE);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        e1.set_value(duel, TYPE_TUNER as i32);
        tc.register_effect(duel, e1);
    }
    Ok(())
}

fn operation1(duel: &mut Duel, a: &Args) -> Script<()> {
    add_tuner(duel, a, |tc, duel| tc.is_faceup(duel))
}

fn filter2(c: CardId, duel: &Duel) -> bool {
    filter1(c, duel) && c.is_race(duel, RACE_FIEND)
}

fn target2(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && filter2(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter2, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, filter2, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    Ok(false)
}

fn operation2(duel: &mut Duel, a: &Args) -> Script<()> {
    add_tuner(duel, a, |tc, duel| tc.is_faceup(duel) && tc.is_race(duel, RACE_FIEND))
}
