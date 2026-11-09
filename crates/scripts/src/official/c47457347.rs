// Omega Summon
use crate::prelude::*;
use crate::testing::{Check, HAND, REMOVED};

const ID: u32 = 47457347;

// In the hand, with Invoked monsters of different names banished: Raidjin, Sorath, Cocytus, Mechaba and Elysium.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (49513164, REMOVED),
    (43989315, REMOVED),
    (85908279, REMOVED),
    (75286621, REMOVED),
    (11270236, REMOVED),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_set_card(duel, &[SET_INVOKED])
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_REMOVED) && chkc.is_controler(duel, tp) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_REMOVED, 0, 1, None));
    }
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ft = 1;
    }
    let g = duel
        .get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_REMOVED, 0, None)
        .filter(duel, |c, duel| c.is_can_be_effect_target(duel, e), None);
    let tg = aux::select_unselect_group(duel, g, 1, g.len().min(ft as usize), &aux::dncheck, tp, HINTMSG_SPSUMMON)?;
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(tg), tg.len() as u8, tp, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    if ft <= 0 || g.is_empty() || (g.len() > 1 && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some()) {
        return Ok(());
    }
    if g.len() as i32 <= ft {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
    } else {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = g.select(duel, tp, ft as u8, ft as u8, None)?;
        duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
        duel.send_to_grave(g.difference(sg), REASON_RULE)?;
    }
    Ok(())
}
