// Lady Labrynth of the Silver Castle
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, SZONE};

const ID: u32 = 81497285;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the hand and on the field, with Normal Traps (Trap Hole Set, Mirror Force in the Deck) to activate in response to and to Set.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (4206964, SZONE),
    (44095762, SZONE),
    (44095762, DECK),
    (44095762, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(CardId::is_facedown, a.e.get_handler_player(duel), LOCATION_ONFIELD, 0, 1, None)
    });
    e1.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e2.set_value(duel, Value::EffectPlayer(aux::indoval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_HAND);
    e3.set_hint_timing(duel, TIMING_END_PHASE, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e3.set_count_limit(duel, 1, ID);
    e3.set_condition(duel, |duel, _| {
        duel.get_custom_activity_count(ID, 0, ACTIVITY_CHAIN) > 0 || duel.get_custom_activity_count(ID, 1, ACTIVITY_CHAIN) > 0
    });
    e3.set_target(duel, sptg);
    e3.set_operation(duel, stock::special_summon_self::<POS_FACEUP_DEFENSE>);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_SET);
    e4.set_type(duel, EFFECT_TYPE_QUICK_O);
    e4.set_code(duel, EVENT_CHAINING);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, (ID, 1));
    e4.set_condition(duel, setcond);
    e4.set_target(duel, settg);
    e4.set_operation(duel, setop);
    c.register_effect(duel, e4);
    duel.add_custom_activity_counter(ID, ACTIVITY_CHAIN, CounterFilter::Chain(checkop));
}

fn checkop(duel: &Duel, re: EffectId, _: u8, _: u8) -> bool {
    let rc = re.get_handler(duel);
    !((rc.is_set_card(duel, &[SET_LABRYNTH]) && !rc.is_code(duel, ID))
        || (re.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && rc.is_normal_trap(duel)))
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned_full(duel, a.e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn setcond(duel: &Duel, a: &Args) -> bool {
    a.re.is_some_and(|re| re.get_active_type(duel) == TYPE_TRAP && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE))
}

fn setfilter(c: CardId, duel: &Duel, code: u32) -> bool {
    c.is_normal_trap(duel) && c.is_ssetable(duel) && !c.is_code(duel, code)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let Some(re) = a.re else { return Ok(false) };
    let code = re.get_handler(duel).get_code(duel);
    Ok(chk == 0 && duel.is_existing_matching_card(|c, duel| setfilter(c, duel, code), a.tp, LOCATION_DECK, 0, 1, None))
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (tp, Some(re)) = (a.tp, a.re) else { return Ok(()) };
    let code = re.get_handler(duel).get_code(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let g = duel.select_matching_card(tp, |c, duel| setfilter(c, duel, code), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.sset(tp, g)?;
    }
    Ok(())
}
