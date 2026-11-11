// The Agent of Destruction - Venus
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, REMOVED};

const ID: u32 = 99054885;
const CREATION_VENUS: u32 = 64734921;
const SHINE_BALL: u32 = 39552864;

pub const LISTED_NAMES: &[u32] = &[CREATION_VENUS, SHINE_BALL];
// In the hand, banishing The Agent of Creation - Venus, and on the field with Mystical Shine Ball in the Graveyard and banished to revive.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (CREATION_VENUS, MZONE),
    (CREATION_VENUS, GRAVE),
    (SHINE_BALL, GRAVE),
    (SHINE_BALL, GRAVE),
    (SHINE_BALL, REMOVED),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
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
    e1.set_cost(duel, hspcost);
    e1.set_target(duel, hsptg);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, spcost);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn hspcostfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_code(duel, CREATION_VENUS)
        && c.is_able_to_remove_as_cost(duel)
        && aux::sp_elim_filter(c, duel, true, false)
        && duel.get_mzone_count(tp, Some(c.into())) > 0
}

fn hspcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let filter = |c, duel: &Duel| hspcostfilter(c, duel, tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn hsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, SHINE_BALL)
        && c.is_faceup(duel)
        && c.is_can_be_effect_target(duel, e)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let sg = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, None);
    let mut ct = sg.len().min(duel.get_location_count(tp, LOCATION_MZONE).max(0) as usize);
    if chk == 0 {
        return Ok(ct > 0 && duel.check_lp_cost(tp, 500));
    }
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ct = 1;
    }
    let max = ct.min(duel.get_lp(tp) as usize / 500);
    let options: Vec<u64> = (1..=max as u64).map(|i| i * 500).collect();
    let ac = duel.announce_number(tp, &options)?;
    duel.pay_lp_cost(tp, ac as u32)?;
    e.set_label(duel, (ac / 500) as i64);
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(true);
    }
    let ct = e.get_label(duel) as u8;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let sg = duel.select_target(
        tp,
        |c, duel| spfilter(c, duel, e, tp),
        tp,
        LOCATION_GRAVE | LOCATION_REMOVED,
        0,
        ct,
        ct,
        None,
    )?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, sg, sg.map_or(0, |sg| sg.len() as u8), tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_target_cards(e).unwrap_or_default();
    if g.is_empty()
        || duel.get_location_count(tp, LOCATION_MZONE) < g.len() as i32
        || (g.len() > 1 && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some())
    {
        return Ok(());
    }
    let c = e.get_handler(duel);
    for sc in g {
        if duel.special_summon_step(sc, 0, tp, tp, false, false, POS_FACEUP)? {
            let e1 = duel.create_effect(c);
            e1.set_description(duel, 3301);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
            e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
            e1.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
            e1.set_value(duel, LOCATION_DECKBOT as i32);
            sc.register_effect_full(duel, e1, true);
        }
    }
    duel.special_summon_complete()?;
    Ok(())
}
