// Duralume, Vaalmonican Heathen Hallow
use super::c2815176::{Procedure, splimit};
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT, PZONE};

const ID: u32 = 76821171;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the Extra Deck, Link Summoned from an Effect Monster once Angello in the Pendulum Zone has 3 Resonance Counters (the Vaalmonica Spells place them), and in
// the Main Monster Zone for its attack effect, with the opponent's monsters to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (3048768, PZONE),
    (30432463, PZONE),
    (65496951, HAND),
    (42193638, HAND),
    (5605529, HAND),
    (78598237, HAND),
    (78598237, GRAVE),
    (26202165, MZONE),
    (26202165, MZONE),
    (26202165, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure<RACE_FAIRY>>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e0.set_value(
        duel,
        Value::SpSummonCondition(|duel, _, _, sp, st, _, _| splimit::<RACE_FAIRY>(duel, sp, st)),
    );
    c.register_effect(duel, e0);
    c.set_sp_summon_once(duel, ID);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, _| duel.is_able_to_enter_bp());
    e2.set_cost(duel, tripleatkcost);
    e2.set_target(duel, tripleatktg);
    e2.set_operation(duel, tripleatkop);
    c.register_effect(duel, e2);
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE));
    }
    let ct = duel.get_counter(tp, 1, 0, COUNTER_RESONANCE);
    if chk == 0 {
        return Ok(ct > 0 && duel.is_existing_target(aux::TRUE, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let tg = duel.select_target(tp, aux::TRUE, tp, 0, LOCATION_MZONE, 1, u8::try_from(ct).unwrap_or(u8::MAX), None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, tg, tg.map_or(0, |tg| tg.len()) as u8, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tg) = duel.get_target_cards(a.e)
        && !tg.is_empty()
    {
        duel.destroy(tg, REASON_EFFECT)?;
    }
    Ok(())
}

fn tripleatkcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_can_remove_counter(tp, 1, 0, COUNTER_RESONANCE, 3, REASON_COST));
    }
    duel.remove_counter(tp, 1, 0, COUNTER_RESONANCE, 3, REASON_COST)?;
    Ok(false)
}

fn tripleatktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk != 0 {
        return Ok(false);
    }
    for eff in a.e.get_handler(duel).is_has_effects(duel, EFFECT_EXTRA_ATTACK) {
        // The Lua compares a function value with a number, which errors.
        let Value::Int(value) = eff.get_value(duel) else {
            unported!(duel, true);
            return Ok(false);
        };
        if value >= 2 {
            return Ok(false);
        }
    }
    Ok(true)
}

fn tripleatkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, aux::stringid(ID, 2));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_EXTRA_ATTACK);
        e1.set_value(duel, 2);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e1);
    }
    Ok(())
}
