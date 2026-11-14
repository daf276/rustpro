// Theorealize Past Lull
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, MZONE};

const ID: u32 = 36709484;

pub const LISTED_SERIES: &[u16] = &[SET_ARTMAGE, SET_DOOMZ, SET_ELFNOTE];
pub const LISTED_NAMES: &[u32] = &[CARD_MEDIUS_THE_PURE];

// Medius the Pure to Special Summon with Elfnote Lucina on the field, a second copy in the Graveyard and I:P Masquerena
// with Chaos Angel in the Graveyard for the Graveyard effect.
pub const SUPPORT: &[(u32, u16)] = &[
    (CARD_MEDIUS_THE_PURE, DECK),
    (13597785, MZONE),
    (ID, GRAVE),
    (65741786, MZONE),
    (22850702, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| {
        let filter = aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_ARTMAGE, SET_DOOMZ, SET_ELFNOTE]));
        duel.is_existing_matching_card(filter, a.tp, LOCATION_ONFIELD, 0, 1, None)
    });
    e1.set_target(duel, mediussptg);
    e1.set_operation(duel, mediusspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, gysptg);
    e2.set_operation(duel, gyspop);
    c.register_effect(duel, e2);
}

fn mediusspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, CARD_MEDIUS_THE_PURE) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn mediussptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let locations = LOCATION_HAND | LOCATION_DECK;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| mediusspfilter(c, duel, e, tp), tp, locations, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, locations as i32);
    Ok(false)
}

fn mediusspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        |c, duel| mediusspfilter(c, duel, e, tp),
        tp,
        LOCATION_HAND | LOCATION_DECK,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() && duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e1.set_code(duel, EVENT_CHAIN_END);
        e1.set_operation(duel, |duel, a| {
            duel.set_chain_limit_till_chain_end(|_, _, rp, tp, _| tp == rp, 0);
            a.e.reset(duel);
            Ok(())
        });
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e1, tp);
    }
    Ok(())
}

fn gyspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, zones: u32) -> bool {
    c.is_type(duel, TYPE_FUSION | TYPE_SYNCHRO | TYPE_XYZ)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, tp, zones)
}

fn gysptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let zones = aux::get_mm_zones_pointed_to(duel, tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && gyspfilter(chkc, duel, e, tp, zones));
    }
    let filter = |c, duel: &Duel| gyspfilter(c, duel, e, tp, zones);
    if chk == 0 {
        return Ok(zones > 0 && duel.is_existing_target(filter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, filter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    Ok(false)
}

fn gyspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let zones = aux::get_mm_zones_pointed_to(duel, tp);
    if zones == 0 {
        return Ok(());
    }
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
    {
        duel.special_summon_full(tc, 0, tp, tp, false, false, POS_FACEUP, zones)?;
    }
    Ok(())
}
