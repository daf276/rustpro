// Harmonizing Magician
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, PZONE};

const ID: u32 = 73941492;

pub const LISTED_SERIES: &[u16] = &[SET_MAGICIAN];

const STARGAZER_MAGICIAN: u32 = 94415058;

pub const LISTED_NAMES: &[u32] = &[ID];

// A scale of 8 in the Pendulum Zone with Stargazer Magician (scale 1) to Pendulum Summon the copy in the hand, and two of them in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (STARGAZER_MAGICIAN, PZONE),
    (ID, HAND),
    (ID, HAND),
    (STARGAZER_MAGICIAN, HAND),
    (STARGAZER_MAGICIAN, DECK),
    (STARGAZER_MAGICIAN, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_value(duel, Value::Card(atkval));
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_SINGLE_RANGE);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_range(duel, LOCATION_EXTRA);
    e3.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e3.set_value(duel, Value::SpSummonCondition(|_, _, _, _, _, _, _| false));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e4.set_type(duel, EFFECT_TYPE_SINGLE);
    e4.set_code(duel, EFFECT_FUSION_MAT_RESTRICTION);
    e4.set_value(duel, Value::Card(|duel, _, c| filter(c, duel).into()));
    c.register_effect(duel, e4);
    let e5 = e4.clone_effect(duel);
    e5.set_code(duel, EFFECT_SYNCHRO_MAT_RESTRICTION);
    c.register_effect(duel, e5);
    let e6 = e4.clone_effect(duel);
    e6.set_code(duel, EFFECT_XYZ_MAT_RESTRICTION);
    c.register_effect(duel, e6);
    let e7 = duel.create_effect(c);
    e7.set_description(duel, aux::stringid(ID, 0));
    e7.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e7.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e7.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e7.set_count_limit(duel, 1, ID);
    e7.set_condition(duel, spcon);
    e7.set_target(duel, sptg);
    e7.set_operation(duel, spop);
    c.register_effect(duel, e7);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MAGICIAN]) && c.is_type(duel, TYPE_PENDULUM)
}

fn atkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && filter(c, duel)
}

fn atkval(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    let g = duel.get_matching_group(atkfilter, c.get_controler(duel), LOCATION_EXTRA, 0, None);
    g.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) as i32 * 100
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.get_summon_type(duel) == SUMMON_TYPE_PENDULUM && c.is_previous_location(duel, LOCATION_HAND)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    filter(c, duel) && !c.is_code(duel, ID) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first()
        && duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?
    {
        let c = e.get_handler(duel);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_DISABLE);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect_full(duel, e1, true);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_code(duel, EFFECT_DISABLE_EFFECT);
        e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect_full(duel, e2, true);
        let e3 = duel.create_effect(c);
        e3.set_description(duel, 3300);
        e3.set_type(duel, EFFECT_TYPE_SINGLE);
        e3.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
        e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e3.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
        e3.set_value(duel, LOCATION_REMOVED as i32);
        tc.register_effect_full(duel, e3, true);
    }
    duel.special_summon_complete()?;
    Ok(())
}
