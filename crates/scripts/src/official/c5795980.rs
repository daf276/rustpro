// Strike of the Monarchs
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 5795980;

// Copies in the hand and Graveyard, and Monarchs to Tribute Summon with Mystical Sheep #2 and Tribute for the cost.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (4929256, HAND),
    (73125233, HAND),
    (51945556, HAND),
    (83464209, MZONE),
    (83464209, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DISABLE + CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e1.set_cost(duel, |duel, a, _| {
        a.e.set_label(duel, 1);
        Ok(true)
    });
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, attg);
    e2.set_operation(duel, atop);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel, dg: CardSet) -> bool {
    if c.is_facedown(duel) || !c.is_level_above(duel, 5) || !c.is_normal_summoned(duel) {
        return false;
    }
    let a = usize::from(dg.contains(c)) + c.get_equip_group(duel).iter().filter(|&tc| dg.contains(tc)).count();
    dg.len() > a
}

fn tgfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_negatable(duel) && c.is_can_be_effect_target(duel, e)
}

fn release_candidates(duel: &Duel, e: EffectId, tp: u8) -> (CardSet, CardSet) {
    let c = e.get_handler(duel);
    let rg = duel.get_release_group(tp);
    let dg = duel.get_matching_group(
        |tc, duel| tgfilter(tc, duel, e),
        tp,
        LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        Some(c.into()),
    );
    (rg, dg)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && tgfilter(chkc, duel, e) && chkc != c);
    }
    if chk == 0 {
        if !duel.is_player_can_draw(tp, 1) {
            return Ok(false);
        }
        if e.get_label(duel) == 1 {
            e.set_label(duel, 0);
            let (rg, dg) = release_candidates(duel, e, tp);
            return Ok(rg.is_exists(duel, |rc, duel| cfilter(rc, duel, dg), 1, Some(c.into())));
        }
        return Ok(duel.is_existing_target(CardId::is_faceup, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(c.into())));
    }
    if e.get_label(duel) == 1 {
        e.set_label(duel, 0);
        let (rg, dg) = release_candidates(duel, e, tp);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_RELEASE);
        let sg = rg.filter_select(duel, tp, |rc, duel| cfilter(rc, duel, dg), 1, 1, Some(c.into()))?;
        duel.release(sg, REASON_COST)?;
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
    let g = duel.select_target(tp, CardId::is_faceup, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, Some(c.into()))?;
    let count = g.map_or(0, |g| g.len() as u8);
    duel.set_operation_info(0, CATEGORY_DISABLE, g, count, 0, 0);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c, tp) = (a.e, a.e.get_handler(duel), a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !(tc.is_relate_to_effect(duel, e) && ((tc.is_faceup(duel) && !tc.is_disabled(duel)) || tc.is_type(duel, TYPE_TRAPMONSTER))) {
        return Ok(());
    }
    duel.negate_related_chain(tc, RESET_TURN_SET);
    let (e1, e2) = super::c97268402::negate(duel, c, tc, 0);
    let e3 = tc.is_type(duel, TYPE_TRAPMONSTER).then(|| {
        let e3 = duel.create_effect(c);
        e3.set_type(duel, EFFECT_TYPE_SINGLE);
        e3.set_code(duel, EFFECT_DISABLE_TRAPMONSTER);
        e3.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e3);
        e3
    });
    if tc.is_immune_to_effect(duel, e1) || tc.is_immune_to_effect(duel, e2) || e3.is_some_and(|e3| tc.is_immune_to_effect(duel, e3)) {
        return Ok(());
    }
    duel.break_effect()?;
    duel.draw(tp, 1, REASON_EFFECT)?;
    Ok(())
}

fn attg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATTRIBUTE);
    let rc = duel.announce_attribute(tp, 1, ATTRIBUTE_ALL)?;
    duel.set_target_param(rc);
    Ok(false)
}

fn atop(duel: &mut Duel, a: &Args) -> Script<()> {
    let rc: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    let g = duel.get_matching_group(CardId::is_faceup, a.tp, LOCATION_MZONE, LOCATION_MZONE, None);
    for tc in g {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CHANGE_ATTRIBUTE);
        e1.set_value(duel, rc as i32);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
