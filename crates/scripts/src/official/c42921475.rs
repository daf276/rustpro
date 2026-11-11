// Fairy Tail - Sleeper
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 42921475;

// Fairy Tail - Luna on the field to Tribute and in the hand to Special Summon, and an opponent's Raigeki and Pot of Greed to
// change.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (86937530, MZONE),
    (86937530, HAND),
    (12580477, HAND | OPPONENT),
    (55144522, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_FLIP + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, chcon);
    e2.set_cost(duel, chcost);
    e2.set_target(duel, chtg);
    e2.set_operation(duel, chop);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_HAND)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_HAND, POS_FACEUP)
}

fn chcon(duel: &Duel, a: &Args) -> bool {
    a.re.is_some_and(|re| a.rp == 1 - a.tp && re.get_handler(duel).is_normal_spell_trap(duel) && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE))
}

fn chcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, aux::TRUE, (1, 1), false, None, Some(c.into())));
    }
    let g = duel.select_release_group_cost(tp, aux::TRUE, (1, 1), false, None, Some(c.into()))?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn chtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_can_turn_set, a.rp, 0, LOCATION_MZONE, 1, None));
    }
    Ok(false)
}

fn chop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.change_target_card(a.ev as u8, CardSet::default());
    duel.change_chain_operation(a.ev as u8, repop);
    Ok(())
}

fn repop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if c.is_normal_spell_trap(duel) {
        c.cancel_to_grave_full(duel, false);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    let g = duel.select_matching_card(tp, CardId::is_can_turn_set, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    if !g.is_empty() {
        duel.change_position(g, POS_FACEDOWN_DEFENSE)?;
    }
    Ok(())
}
