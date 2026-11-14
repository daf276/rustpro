// Goblin Biker Grand Breakout
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 29111045;

pub const LISTED_SERIES: &[u16] = &[SET_GOBLIN];

const GOBLIN_BIKERS_GONE_BONKERS: u32 = 46014517;

// In hand with Goblin Zombie and Goblin Attack Force in the Deck to summon and Goblin Bikers Gone Bonkers on the field to
// detach from, and a copy in the Graveyard against Sangan of the opponent's.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (63665875, DECK),
    (78658564, DECK),
    (GOBLIN_BIKERS_GONE_BONKERS, MZONE),
    (GOBLIN_BIKERS_GONE_BONKERS, MZONE),
    (26202165, MZONE | OPPONENT),
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
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, spcost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_ATKCHANGE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, atkcost);
    e2.set_target(duel, atktg);
    e2.set_operation(duel, atkop);
    c.register_effect(duel, e2);
}

fn spcostfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    duel.get_mzone_count(tp, Some(c.into())) > 0
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    a.e.set_label(duel, 1);
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, |c, duel| spcostfilter(c, duel, tp), (1, 1), false, None, None));
    }
    let sg = duel.select_release_group_cost(tp, |c, duel| spcostfilter(c, duel, tp), (1, 1), false, None, None)?;
    duel.release(sg, REASON_COST)?;
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_GOBLIN]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let cost_chk = e.get_label(duel) == 1;
        e.set_label(duel, 0);
        return Ok((cost_chk || duel.get_location_count(tp, LOCATION_MZONE) > 0)
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first()
        && duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)? > 0
    {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_description(duel, 3206);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_CANNOT_ATTACK);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect_full(duel, e1, true);
    }
    Ok(())
}

fn atkcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GOBLIN]) && c.is_type(duel, TYPE_XYZ) && c.is_faceup(duel) && c.get_overlay_count(duel) > 0
}

fn atkcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    e.set_label(duel, -1);
    let c = e.get_handler(duel);
    let xyzg = duel.get_matching_group(atkcostfilter, tp, LOCATION_MZONE, 0, None);
    if chk == 0 {
        return Ok(c.is_able_to_remove_as_cost(duel)
            && !xyzg.is_empty()
            && duel.check_remove_overlay_card_full(tp, 0, 0, 1, REASON_COST, Some(xyzg)));
    }
    duel.remove(c, POS_FACEUP, REASON_COST)?;
    let maxct = xyzg.get_sum(duel, |c, duel| c.get_overlay_count(duel) as i64) as u16;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVEXYZ);
    let ct = duel.remove_overlay_card_full(tp, 0, 0, 1, maxct, REASON_COST, Some(xyzg))?;
    e.set_label(duel, i64::from(ct));
    Ok(false)
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let atkg = duel.get_matching_group(CardId::is_faceup, tp, 0, LOCATION_MZONE, None);
    if chk == 0 {
        let cost_chk = e.get_label(duel) == -1;
        e.set_label(duel, 0);
        return Ok(cost_chk && !atkg.is_empty());
    }
    let atk = (e.get_label(duel) * -1000) as i32;
    duel.set_operation_info(0, CATEGORY_ATKCHANGE, Some(atkg), atkg.len() as u8, tp, atk);
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(CardId::is_faceup, tp, 0, LOCATION_MZONE, None);
    let c = e.get_handler(duel);
    let atk = (e.get_label(duel) * -1000) as i32;
    for tc in g {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, atk);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
