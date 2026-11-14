// Traptrip Garden
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 12801833;

pub const LISTED_SERIES: &[u16] = &[SET_TRAPTRIX];
// Traptrix Myrmeleo and Pudica in the hand and GY to Special Summon, and a monster on the field to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (91812341, HAND),
    (91812341, GRAVE),
    (49027020, GRAVE),
    (91812341, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_EXTRA_SUMMON_COUNT);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_target_range(duel, LOCATION_HAND | LOCATION_MZONE, 0);
    e2.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_TRAPTRIX]));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_COUNT);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_target_range(duel, LOCATION_MZONE, 0);
    e3.set_target_card(duel, |duel, _, c| c.is_race(duel, RACE_INSECT | RACE_PLANT));
    e3.set_value(duel, Value::Reason(|_, _, _, r, _| r & REASON_BATTLE == REASON_BATTLE));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_FZONE);
    e4.set_count_limit(duel, 1, ID);
    e4.set_cost(duel, spcost);
    e4.set_target(duel, sptg);
    e4.set_operation(duel, spop);
    c.register_effect(duel, e4);
}

fn spcostfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_able_to_remove_as_cost(duel) && duel.get_mzone_count(tp, Some(c.into())) > 0
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    stock::banish_cost(duel, a, chk, |c, duel| spcostfilter(c, duel, tp), LOCATION_MZONE)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_TRAPTRIX]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let filter = |c, duel: &Duel| spfilter(c, duel, e, tp);
    stock::exists_target(duel, a, chk, filter, LOCATION_HAND | LOCATION_GRAVE, CATEGORY_SPECIAL_SUMMON)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) == 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
