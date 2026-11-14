// Blackwing - Auster the South Wind
use engine::Value;

use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT, REMOVED};

const ID: u32 = 17465972;

pub const LISTED_SERIES: &[u16] = &[SET_BLACKWING];
pub const COUNTER_PLACE_LIST: &[u16] = &[0x1002, COUNTER_FEATHER];
const WEDGE_COUNTER: u16 = 0x1002;

pub const LISTED_NAMES: &[u32] = &[CARD_BLACK_WINGED_DRAGON];

// In hand and in the Graveyard, with a banished Gale to Special Summon and a monster of the opponent's to place a counter on.
// The Black-Winged Dragon option is out of reach: it is not ported.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (2009101, REMOVED),
    (2009101, REMOVED),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_value(duel, Value::SpSummonCondition(|_, _, _, _, _, _, _| false));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_category(duel, CATEGORY_COUNTER);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_cost(duel, Cost::self_banish);
    e3.set_target(duel, cttg);
    c.register_effect(duel, e3);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_level_below(duel, 4)
        && c.is_set_card(duel, &[SET_BLACKWING])
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_REMOVED) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_REMOVED, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_REMOVED, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
    {
        duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
    }
    Ok(())
}

fn wcfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.get_counter(duel, WEDGE_COUNTER) == 0
}

fn cttg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let b1 = duel.is_existing_matching_card(
        aux::faceup_filter(|c, duel| c.is_code(duel, CARD_BLACK_WINGED_DRAGON)),
        tp,
        LOCATION_MZONE,
        0,
        1,
        None,
    ) && duel.get_field_group_count(tp, 0, LOCATION_ONFIELD) > 0;
    let b2 = duel.is_existing_matching_card(wcfilter, tp, 0, LOCATION_MZONE, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = if b1 && b2 {
        duel.select_option(tp, &[aux::stringid(ID, 2), aux::stringid(ID, 3)])?
    } else if b1 {
        duel.select_option(tp, &[aux::stringid(ID, 2)])?
    } else {
        duel.select_option(tp, &[aux::stringid(ID, 3)])? + 1
    };
    a.e.set_operation(duel, if op == 0 { bwop } else { wcop });
    Ok(false)
}

fn bwop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    let tc = duel
        .select_matching_card(
            tp,
            aux::faceup_filter(|c, duel| c.is_code(duel, CARD_BLACK_WINGED_DRAGON)),
            tp,
            LOCATION_MZONE,
            0,
            1,
            1,
            None,
        )?
        .get_first();
    let ct = duel.get_field_group_count(tp, 0, LOCATION_ONFIELD);
    if let Some(tc) = tc
        && ct > 0
    {
        tc.add_counter(duel, COUNTER_FEATHER, ct as u16);
    }
    Ok(())
}

fn wcop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(wcfilter, a.tp, 0, LOCATION_MZONE, None);
    for tc in g.iter() {
        tc.add_counter(duel, WEDGE_COUNTER, 1);
    }
    Ok(())
}
