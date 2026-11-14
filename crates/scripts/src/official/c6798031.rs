// Ryzeal Cross
use engine::constants::MSG_SELECT_EFFECTYN;

use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 6798031;

pub const LISTED_SERIES: &[u16] = &[SET_RYZEAL];
pub const LISTED_NAMES: &[u32] = &[ID];

// In the hand with Ryzeal Detonator and Ryzeal monsters in the Graveyard, and Raigeki to send the opponent's Sangan to the
// Graveyard for its effect to negate. The negation is no chain link; its prompt is the MSG_SELECT_EFFECTYN, counted for any card.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (34909328, MZONE),
    (8633261, GRAVE),
    (35844557, GRAVE),
    (34022970, GRAVE),
    (26202165, MZONE | OPPONENT),
    (12580477, HAND),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Message(MSG_SELECT_EFFECTYN)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_range(duel, LOCATION_FZONE);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, xyzlimit);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TODECK + CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, drtg);
    e2.set_operation(duel, drop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EVENT_CHAIN_SOLVING);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_condition(duel, negcon);
    e3.set_operation(duel, negop);
    c.register_effect(duel, e3);
}

fn xyzlimit(duel: &Duel, _: EffectId, c: CardId, sump: u8, sumtype: u32, _: u8, _: u8, _: Option<EffectId>) -> bool {
    sumtype & SUMMON_TYPE_XYZ == SUMMON_TYPE_XYZ
        && duel.is_existing_matching_card(
            aux::faceup_filter(|tc, duel| tc.is_code(duel, c.get_code(duel))),
            sump,
            LOCATION_ONFIELD,
            0,
            1,
            None,
        )
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RYZEAL]) && c.is_able_to_deck(duel) && !c.is_code(duel, ID)
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 1) && duel.is_existing_target(tdfilter, tp, LOCATION_GRAVE, 0, 2, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, tdfilter, tp, LOCATION_GRAVE, 0, 2, 2, None)?;
    duel.set_operation_info(0, CATEGORY_TODECK, g, 2, tp, 0);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    Ok(false)
}

fn drop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if tg.is_empty() || duel.send_to_deck(tg, PLAYER_NONE, SEQ_DECKBOTTOM, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let dg = duel
        .get_operated_group()
        .filter(duel, |c, duel| c.is_location(duel, LOCATION_DECK | LOCATION_EXTRA), None);
    if dg.is_empty() {
        return Ok(());
    }
    let ct = dg.filter_count(duel, |c, duel| c.is_location(duel, LOCATION_DECK), None);
    if ct > 1 {
        duel.sort_deckbottom(tp, tp, ct as u16)?;
    }
    if duel.is_player_can_draw(tp, 0) {
        duel.break_effect()?;
        duel.draw(tp, 1, REASON_EFFECT)?;
    }
    Ok(())
}

fn xyzfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RYZEAL]) && c.is_type(duel, TYPE_XYZ) && c.get_overlay_count(duel) > 0 && c.is_faceup(duel)
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    let (tp, ev) = (a.tp, a.ev as u8);
    let g = duel.get_matching_group(xyzfilter, tp, LOCATION_MZONE, 0, None);
    a.rp == 1 - tp
        && a.re.is_some_and(|re| re.is_monster_effect(duel))
        && duel.is_chain_disablable(ev)
        && !a.e.get_handler(duel).has_flag_effect(duel, ID)
        && !g.is_empty()
        && duel.check_remove_overlay_card_full(tp, 1, 0, 1, REASON_EFFECT, Some(g))
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp, ev) = (a.e.get_handler(duel), a.tp, a.ev as u8);
    let g = duel.get_matching_group(xyzfilter, tp, LOCATION_MZONE, 0, None);
    let cid: i64 = duel.get_chain_info(ev, CHAININFO_CHAIN_ID);
    if !(duel.get_flag_effect_label(tp, ID) != Some(cid) && !g.is_empty() && duel.select_effect_yes_no(tp, c)?) {
        return Ok(());
    }
    duel.hint(HINT_CARD, 0, u64::from(ID));
    duel.remove_overlay_card_full(tp, 1, 0, 1, 1, REASON_EFFECT, Some(g))?;
    duel.negate_effect(ev);
    c.register_flag_effect_full(
        duel,
        ID,
        RESETS_STANDARD_PHASE_END,
        EFFECT_FLAG_CLIENT_HINT,
        1,
        0,
        aux::stringid(ID, 1),
    );
    duel.register_flag_effect_full(tp, ID, RESET_CHAIN, 0, 1, cid as u32);
    Ok(())
}
