// Tuning Gum
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 82744076;

// On the field and in the Graveyard with a Synchro Monster (Magical Android) on the field, targeted by the opponent's Mage Power
// and Mind Control in hand.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, GRAVE),
    (43385557, MZONE),
    (83746708, HAND | OPPONENT),
    (37520316, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_NEGATE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_condition(duel, negcon);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, negtg);
    e2.set_operation(duel, negop);
    c.register_effect(duel, e2);
    duel.add_custom_activity_counter(ID, ACTIVITY_ATTACK, CounterFilter::Card(counterfilter));
}

fn counterfilter(duel: &Duel, c: CardId) -> bool {
    c.is_type(duel, TYPE_SYNCHRO)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_custom_activity_count(ID, tp, ACTIVITY_ATTACK) == 0);
    }
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK);
    e1.set_property(duel, EFFECT_FLAG_OATH + EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_target_card(duel, |duel, _, c| !c.is_type(duel, TYPE_SYNCHRO));
    e1.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(false)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && !c.is_type(duel, TYPE_TUNER)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, filter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && tc.is_faceup(duel)
    {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_ADD_TYPE);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        e1.set_value(duel, TYPE_TUNER as i32);
        tc.register_effect(duel, e1);
    }
    Ok(())
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_controler(duel, tp) && c.is_location(duel, LOCATION_MZONE) && c.is_faceup(duel) && c.is_type(duel, TYPE_SYNCHRO)
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    if !re.is_has_property(duel, EFFECT_FLAG_CARD_TARGET) {
        return false;
    }
    let g: Option<CardSet> = duel.get_chain_info(a.ev as u8, CHAININFO_TARGET_CARDS);
    let Some(g) = g.filter(|g| g.len() == 1) else { return false };
    g.get_first().is_some_and(|tc| cfilter(tc, duel, a.tp)) && duel.is_chain_negatable(a.ev as u8)
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.negate_activation(a.ev as u8)
        && let Some(re) = a.re
        && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.send_to_grave(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}
