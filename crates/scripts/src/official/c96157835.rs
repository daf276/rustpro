// Raidraptor - Arsenal Falcon
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE};

const ID: u32 = 96157835;

pub const LISTED_SERIES: &[u16] = &[SET_RAIDRAPTOR];
pub const LISTED_NAMES: &[u32] = &[ID];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(7);
    const CT: usize = 2;
}

// Level 4 Winged Beast Normal Monsters (Sky Scout, Skull Red Bird) in the Deck to Special Summon. A Raidraptor material only
// gets attached by a Rank-Up, so the Graveyard trigger is only reported.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (ID, MZONE), (30532390, DECK), (10202894, DECK)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, sptg1);
    e1.set_operation(duel, spop1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_EXTRA_ATTACK);
    e2.set_condition(duel, ctcon);
    e2.set_value(duel, Value::Card(ctval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_condition(duel, spcon2);
    e3.set_target(duel, sptg2);
    e3.set_operation(duel, spop2);
    c.register_effect(duel, e3);
}

fn spfilter1(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_WINGEDBEAST) && c.get_level(duel) == 4 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg1(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter1(c, duel, a.e, a.tp), LOCATION_DECK)
}

fn spop1(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| spfilter1(c, duel, a.e, a.tp), LOCATION_DECK, POS_FACEUP)
}

fn ctcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel)
        .get_overlay_group(duel)
        .is_exists(duel, |c, duel| c.is_set_card(duel, &[SET_RAIDRAPTOR]), 1, None)
}

fn ctval(duel: &Duel, e: EffectId, _: CardId) -> i32 {
    e.get_handler(duel)
        .get_overlay_group(duel)
        .filter_count(duel, |c, duel| c.is_set_card(duel, &[SET_RAIDRAPTOR]), None) as i32
        - 1
}

fn spcon2(duel: &Duel, a: &Args) -> bool {
    ctcon(duel, a) && a.e.get_handler(duel).is_previous_location(duel, LOCATION_MZONE)
}

fn spfilter2(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_RAIDRAPTOR])
        && c.is_type(duel, TYPE_XYZ)
        && !c.is_code(duel, ID)
        && duel.get_location_count_from_ex(tp, tp, None, c) > 0
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg2(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| spfilter2(c, duel, e, tp), tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(e.get_handler(duel).into()), 1, 0, 0);
    Ok(false)
}

fn spop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter2(c, duel, e, tp), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first()
        && duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)? > 0
    {
        let c = e.get_handler(duel);
        if c.is_relate_to_effect(duel, e) {
            duel.overlay(tc, c)?;
        }
    }
    Ok(())
}
