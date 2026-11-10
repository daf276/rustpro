// Dragonmaid Cehrmba
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, REMOVED};

const ID: u32 = 12163590;
const KITCHEN_DRAGONMAID: u32 = 16960120;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the hand and on the field, with Kitchen Dragonmaid in the hand, Graveyard and banishment.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (KITCHEN_DRAGONMAID, HAND),
    (KITCHEN_DRAGONMAID, GRAVE),
    (KITCHEN_DRAGONMAID, REMOVED),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_race(duel, RACE_DRAGON) && c.is_type(duel, TYPE_FUSION));
    e1.set_value(duel, Value::EffectPlayer(aux::indoval));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, Cost::self_discard);
    e2.set_target(duel, gyrmsptg);
    e2.set_operation(duel, gyrmspop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_PHASE | PHASE_BATTLE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
}

fn gyrmspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_DRAGONMAID])
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && c.is_faceup(duel)
        && !c.is_code(duel, ID)
}

fn gyrmsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let filter = |c, duel: &Duel| gyrmspfilter(c, duel, e, tp);
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(filter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_GRAVE | LOCATION_REMOVED) as i32);
    Ok(false)
}

fn gyrmspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = |c, duel: &Duel| gyrmspfilter(c, duel, e, tp);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(filter),
        tp,
        LOCATION_GRAVE | LOCATION_REMOVED,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn handspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_below(duel, 4) && c.is_set_card(duel, &[SET_DRAGONMAID]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        let filter = |sc, duel: &Duel| handspfilter(sc, duel, e, tp);
        return Ok(duel.get_mzone_count(tp, Some(c.into())) > 0
            && c.is_able_to_hand(duel)
            && duel.is_existing_matching_card(filter, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e)
        && duel.send_to_hand(c, PLAYER_NONE, REASON_EFFECT)? > 0
        && c.is_location(duel, LOCATION_HAND)
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
    {
        duel.shuffle_hand(tp)?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let filter = |sc, duel: &Duel| handspfilter(sc, duel, e, tp);
        let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
