// Pillar of the Future - Cyanos
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, REMOVED};

const ID: u32 = 20357457;

pub const LISTED_SERIES: &[u16] = &[SET_SKY_STRIKER];
pub const LISTED_NAMES: &[u32] = &[37351133];
// Spells in hand to discard, Sky Striker Ace - Roze in the Deck, Graveyard and banished, and a copy in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (75652080, HAND),
    (75652080, HAND),
    (37351133, DECK),
    (37351133, DECK),
    (37351133, GRAVE),
    (37351133, REMOVED),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_cost(duel, selfspcost);
    e1.set_target(duel, selfsptg);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, rozesptg);
    e2.set_operation(duel, rozespop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_category(duel, CATEGORY_TOHAND + CATEGORY_SPECIAL_SUMMON);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_GRAVE);
    e4.set_count_limit(duel, 1, (ID, 2));
    e4.set_cost(duel, Cost::self_banish);
    e4.set_target(duel, thsptg);
    e4.set_operation(duel, thspop);
    c.register_effect(duel, e4);
}

fn selfspcfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell(duel) && c.is_discardable(duel)
}

fn selfspcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(selfspcfilter, tp, LOCATION_HAND, 0, 1, Some(a.e.get_handler(duel).into())));
    }
    duel.discard_hand(tp, selfspcfilter, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn selfsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn rozespfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, 37351133) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn rozesptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let location = LOCATION_DECK | LOCATION_GRAVE;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| rozespfilter(c, duel, e, tp), tp, location, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, location as i32);
    Ok(false)
}

fn rozespop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 3));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| {
        !c.is_race(duel, RACE_MACHINE) && c.is_location(duel, LOCATION_EXTRA)
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| !c.is_original_race(duel, RACE_MACHINE));
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| rozespfilter(c, duel, e, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn thspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, ft: i32) -> bool {
    c.is_code(duel, 37351133)
        && c.is_faceup(duel)
        && (c.is_able_to_hand(duel) || (ft > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)))
}

fn thsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    let location = LOCATION_REMOVED | LOCATION_GRAVE;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| thspfilter(c, duel, e, tp, ft), tp, location, 0, 1, None));
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, location as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, location as i32);
    Ok(false)
}

fn thspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| thspfilter(c, duel, e, tp, ft));
    let Some(sc) = duel
        .select_matching_card(tp, filter, tp, LOCATION_REMOVED | LOCATION_GRAVE, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    aux::to_hand_or_else(
        duel,
        sc,
        tp,
        |sc, duel| ft > 0 && sc.is_can_be_special_summoned(duel, e, 0, tp, false, false),
        |duel| duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP).map(drop),
        aux::stringid(ID, 4),
    )
}
