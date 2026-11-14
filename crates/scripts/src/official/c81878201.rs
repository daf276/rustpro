// Abyss-sting Triaina
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, OPPONENT};

const ID: u32 = 81878201;

pub const LISTED_SERIES: &[u16] = &[SET_ATLANTEAN, SET_MERMAIL];
// Atlantean Pikeman in the hand and GY to summon and to shuffle back, with the opponent's Dark Hole to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (26976414, HAND),
    (26976414, GRAVE),
    (26976414, GRAVE),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_EQUIP);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_EQUIP + EFFECT_TYPE_CONTINUOUS);
    e2.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e2.set_code(duel, EFFECT_DESTROY_REPLACE);
    e2.set_target(duel, replacetg);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TODECK);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_cost(duel, Cost::self_banish);
    e3.set_target(duel, tdtg);
    e3.set_operation(duel, tdop);
    c.register_effect(duel, e3);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, LISTED_SERIES) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_GRAVE) as i32);
    let c = e.get_handler(duel);
    duel.set_operation_info(0, CATEGORY_EQUIP, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) == 0 {
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
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)? > 0 && duel.equip(tp, c, tc)? {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_EQUIP_LIMIT);
        e1.set_value(duel, Value::Card(super::c38423248::equip_limit));
        e1.set_label_object_group(duel, Some(tc.into()));
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn replacetg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    let Some(ec) = c.get_equip_target(duel) else { return Ok(false) };
    if chk == 0 {
        return Ok(ec.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
            && !ec.is_reason(duel, REASON_REPLACE)
            && c.is_able_to_grave(duel)
            && !c.is_status(duel, STATUS_DESTROY_CONFIRMED));
    }
    if duel.select_effect_yes_no_full(a.tp, c, 96)? {
        duel.send_to_grave(c, REASON_EFFECT)?;
        return Ok(true);
    }
    Ok(false)
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_FISH | RACE_SEASERPENT | RACE_AQUA) && c.is_faceup(duel) && c.is_able_to_deck(duel)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE | LOCATION_REMOVED) && tdfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(tdfilter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, tdfilter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, 3, None)?;
    let count = g.map_or(0, |g| g.len() as u8);
    duel.set_operation_info(0, CATEGORY_TODECK, g, count, tp, 0);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tg) = duel.get_target_cards(a.e)
        && !tg.is_empty()
    {
        duel.send_to_deck(tg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    }
    Ok(())
}
