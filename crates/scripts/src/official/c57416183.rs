// Dragonmaid Tidying
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, SZONE};

const ID: u32 = 57416183;

pub const LISTED_SERIES: &[u16] = &[SET_DRAGONMAID];

const KITCHEN_DRAGONMAID: u32 = 16960120;

// Set, with Kitchen Dragonmaid on the field to return and in the hand and GY to Special Summon; in the GY to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, GRAVE),
    (KITCHEN_DRAGONMAID, MZONE),
    (KITCHEN_DRAGONMAID, MZONE),
    (KITCHEN_DRAGONMAID, HAND),
    (KITCHEN_DRAGONMAID, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, ID);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_race(duel, RACE_DRAGON) && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_MZONE, 0, 1, None)
            && duel.is_existing_target(CardId::is_able_to_hand, tp, 0, LOCATION_ONFIELD | LOCATION_GRAVE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let mut g = duel
        .select_target(tp, filter, tp, LOCATION_MZONE, 0, 1, 1, None)?
        .unwrap_or_default();
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g2 = duel.select_target(tp, CardId::is_able_to_hand, tp, 0, LOCATION_ONFIELD | LOCATION_GRAVE, 1, 1, None)?;
    g.merge(g2.unwrap_or_default());
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(g), 2, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel
        .get_chain_info::<Option<CardSet>>(0, CHAININFO_TARGET_CARDS)
        .unwrap_or_default()
        .filter(duel, |c, duel| c.is_relate_to_effect(duel, a.e), None);
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_DRAGONMAID]) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let filter = |c, duel: &Duel| spfilter(c, duel, e, tp);
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(filter, tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    let Some(tc) = duel
        .select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
    let fid = tc.get_field_id(duel);
    tc.register_flag_effect_full(duel, ID, RESET_EVENT | RESETS_STANDARD, 0, 1, fid.into(), 0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_PHASE + PHASE_END);
    e1.set_count_limit(duel, 1, 0);
    e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_label(duel, fid.into());
    e1.set_label_object_group(duel, Some(tc.into()));
    e1.set_condition_mut(duel, thcon);
    e1.set_operation(duel, thop);
    duel.register_effect(e1, tp);
    Ok(())
}

fn thcon(duel: &mut Duel, a: &Args) -> bool {
    let tc = a.e.get_label_object_group(duel).unwrap_or_default();
    let label = a.e.get_label(duel);
    if tc.get_first().and_then(|tc| tc.get_flag_effect_label(duel, ID)) != Some(label) {
        a.e.reset(duel);
        return false;
    }
    true
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tc = a.e.get_label_object_group(duel).unwrap_or_default();
    duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
    Ok(())
}
