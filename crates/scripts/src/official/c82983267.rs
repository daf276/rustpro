// Purrelyeap!?
use crate::keys;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, SZONE};

const ID: u32 = 82983267;

pub const LISTED_SERIES: &[u16] = &[SET_PURRELY];

// Set, with Epurrely Plump on the field to become Expurrely Happiness, and Epurrelys in the Graveyard to shuffle back.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, GRAVE),
    (24434049, MZONE),
    (24434049, MZONE),
    (51822687, EXTRA),
    (51822687, EXTRA),
    (62592805, GRAVE),
    (98049934, GRAVE),
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
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TODECK);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, tdtg);
    e2.set_operation(duel, tdop);
    c.register_effect(duel, e2);
}

fn must_be_material_ok(duel: &Duel, c: CardId, tp: u8) -> bool {
    let pg = aux::get_must_be_material_group(duel, tp, Some(c.into()), tp, None, None, REASON_XYZ);
    pg.is_empty() || (pg.len() == 1 && pg.contains(c))
}

fn sptgfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    if c.is_facedown(duel) || !c.is_set_card(duel, &[SET_PURRELY]) {
        return false;
    }
    must_be_material_ok(duel, c, tp)
        && duel.is_existing_matching_card(|sc, duel| spfilter(sc, duel, e, tp, c), tp, LOCATION_EXTRA, 0, 1, None)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: CardId) -> bool {
    mc.is_type_full(duel, TYPE_XYZ, Some(c), SUMMON_TYPE_XYZ.into(), tp)
        && c.is_type(duel, TYPE_XYZ)
        && c.is_set_card(duel, &[SET_PURRELY])
        && duel.get_location_count_from_ex(tp, tp, Some(mc.into()), c) > 0
        && !c.is_rank(duel, &[mc.get_rank(duel)])
        && mc.is_can_be_xyz_material(duel, Some(c), tp, REASON_EFFECT)
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_XYZ, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && sptgfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| sptgfilter(c, duel, e, tp), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, |c, duel| sptgfilter(c, duel, e, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

struct Return;

impl aux::DelayedOperation for Return {
    fn operate(duel: &mut Duel, ag: CardSet, _: &Args) -> Script<()> {
        duel.send_to_deck(ag, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
        Ok(())
    }

    fn condition(duel: &Duel, _: CardSet, a: &Args) -> bool {
        i64::from(duel.get_turn_count()) == duel.get_script_state(keys::delayed_turn(ID, a.e.get_label(duel))) + 1
    }
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    let reason_player = duel.get_reason_player();
    if !(tc.is_faceup(duel)
        && tc.is_relate_to_effect(duel, e)
        && tc.is_controler(duel, tp)
        && tc.is_can_be_xyz_material(duel, None, reason_player, REASON_XYZ | REASON_MATERIAL)
        && !tc.is_immune_to_effect(duel, e))
    {
        return Ok(());
    }
    if !must_be_material_ok(duel, tc, tp) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp, tc), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    let Some(sc) = g.get_first() else { return Ok(()) };
    sc.set_material(duel, Some(tc.into()));
    duel.overlay(sc, tc)?;
    if duel.special_summon_step(sc, SUMMON_TYPE_XYZ, tp, tp, false, false, POS_FACEUP)? {
        let turn_ct = i64::from(duel.get_turn_count());
        let reset = RESET_PHASE | PHASE_END;
        let hint = aux::stringid(ID, 2);
        if let Some(delayed) = aux::delayed_operation_full::<Return>(duel, sc.into(), PHASE_END, reset, ID, e, tp, hint, 2) {
            let key = keys::delayed_turn(ID, delayed.get_label(duel));
            duel.set_script_state(key, turn_ct);
        }
    }
    if duel.special_summon_complete()? == 0 {
        return Ok(());
    }
    sc.complete_procedure(duel);
    Ok(())
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_set_card(duel, &[SET_PURRELY]) && c.is_able_to_deck(duel)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && tdfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(tdfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, tdfilter, tp, LOCATION_GRAVE, 0, 1, 3, None)?;
    let ct = g.map_or(0, |g| g.len() as u8);
    duel.set_operation_info(0, CATEGORY_TODECK, g, ct, 0, 0);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if !tg.is_empty() {
        duel.send_to_deck(tg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    }
    Ok(())
}
