// Auram the World Chalice Blademaster
use super::c31226177::{spcon2, spop2, sptg2};
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 4709881;

pub const LISTED_SERIES: &[u16] = &[SET_WORLD_LEGACY, SET_WORLD_CHALICE];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_WORLD_CHALICE], Some(lc), sumtype, tp)
    }
}

// In the Extra Deck, Link Summoned with Lee the World Chalice Fairy (which then fills the Graveyard), with "World Legacy" monsters in
// the Graveyard for ATK, Lee in the hand to Special Summon, and Raigeki from the opponent's hand to send it to the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (21893603, MZONE),
    (21893603, MZONE),
    (21893603, MZONE),
    (21893603, HAND),
    (21893603, GRAVE),
    (27918365, GRAVE),
    (93920420, GRAVE),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::Card(|duel, _, c| atkval(duel, c.get_controler(duel))));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, spcost1);
    e2.set_target(duel, sptg1);
    e2.set_operation(duel, spop1);
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

fn atkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_set_card(duel, &[SET_WORLD_LEGACY])
}

fn atkval(duel: &Duel, player: u8) -> i32 {
    duel.get_matching_group(atkfilter, player, LOCATION_GRAVE, 0, None)
        .get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) as i32
        * 300
}

fn cfilter(c: CardId, duel: &Duel, g: CardSet, tp: u8, zone: u32) -> bool {
    c.is_set_card(duel, &[SET_WORLD_CHALICE])
        && g.contains(c)
        && duel.get_mzone_count_full(tp, Some(c.into()), tp, LOCATION_REASON_TOFIELD, zone) > 0
}

fn spcost1(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    let lg = c.get_linked_group(duel);
    let zone = c.get_linked_zone_full(duel, tp);
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, |c, duel| cfilter(c, duel, lg, tp, zone), (1, 1), false, None, None));
    }
    let g = duel.select_release_group_cost(tp, |c, duel| cfilter(c, duel, lg, tp, zone), (1, 1), false, None, None)?;
    duel.release(g, REASON_COST)?;
    a.e.set_label_object_group(duel, Some(g));
    Ok(false)
}

fn spfilter1(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg1(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let cc = e.get_label_object_group(duel).and_then(|g| g.first());
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE)
            && chkc.is_controler(duel, tp)
            && Some(chkc) != cc
            && spfilter1(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| spfilter1(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, cc.map(Into::into)));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(
        tp,
        |c, duel| spfilter1(c, duel, e, tp),
        tp,
        LOCATION_GRAVE,
        0,
        1,
        1,
        cc.map(Into::into),
    )?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn spop1(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let zone = a.e.get_handler(duel).get_linked_zone_full(duel, tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.special_summon_full(tc, 0, tp, tp, false, false, POS_FACEUP, zone)?;
    }
    Ok(())
}
