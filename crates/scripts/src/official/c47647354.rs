// Rex, Ride Fur Hire
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 47647354;

pub const LISTED_SERIES: &[u16] = &[SET_FUR_HIRE];
pub const LISTED_NAMES: &[u32] = &[ID];
// In the hand with a second Fur Hire to Special Summon, on the field with one to draw when the opponent's Pot of Greed does.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (93850652, HAND),
    (ID, MZONE),
    (93850652, MZONE),
    (55144522, HAND | OPPONENT),
    (55144522, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::self_reveal);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_HAND);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_CHAIN);
    e2.set_condition(duel, drwcon);
    e2.set_target(duel, drwtg);
    e2.set_operation(duel, stock::draw);
    c.register_effect(duel, e2);
}

// Shared with the Fur Hire monsters that Special Summon another from the hand.
pub fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, code: u32) -> bool {
    c.is_set_card(duel, &[SET_FUR_HIRE]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false) && !c.is_code(duel, code)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) >= 2
            && duel.is_player_can_special_summon_count(tp, 2)
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp, ID), tp, LOCATION_HAND, 0, 1, None)
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 2, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    let c = e.get_handler(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp, ID), tp, LOCATION_HAND, 0, 1, 1, None)?;
    if !g.is_empty() && duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)? > 0 && c.is_relate_to_effect(duel, e) {
        duel.break_effect()?;
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn drwconfilter(c: CardId, duel: &Duel, opp: u8) -> bool {
    c.is_previous_location(duel, LOCATION_DECK) && c.is_controler(duel, opp) && c.is_reason(duel, REASON_EFFECT)
}

fn drwcon(duel: &Duel, a: &Args) -> bool {
    a.re.is_some_and(|re| !re.get_handler(duel).is_set_card(duel, &[SET_FUR_HIRE]))
        && a.eg
            .unwrap_or_default()
            .is_exists(duel, |c, duel| drwconfilter(c, duel, 1 - a.tp), 1, None)
}

fn drwtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_FUR_HIRE])),
            tp,
            LOCATION_MZONE,
            0,
            1,
            Some(a.e.get_handler(duel).into()),
        ) && duel.is_player_can_draw(tp, 1));
    }
    duel.set_target_player(tp);
    duel.set_target_param(1);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    Ok(false)
}
