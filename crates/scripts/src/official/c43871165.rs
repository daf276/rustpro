// Jupredo the Shademachine Power Patron
// The script's spfilter, spcon, sptg and spop belong to no effect and are left out.
use super::c10266279::mdescost;
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, SZONE};

const ID: u32 = 43871165;
const JUPITER: u32 = 68231287;

pub const LISTED_NAMES: &[u32] = &[JUPITER];

// In the Pendulum Zone with Elfnote Power Patron in the hand to destroy with it, and a "Power Patron" Spell in the Deck for
// the effect when it is added to the Extra Deck. Jupiter the Power Patron of Destruction is not ported, so the Xyz Summon
// effect only reports.
pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE), (12375297, HAND), (25661743, DECK)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, pdestg);
    e1.set_operation(duel, pdesop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, |duel, a, chk| Cost::replaceable(duel, a, chk, mdescost));
    e2.set_target(duel, mdestg);
    e2.set_operation(duel, mdesop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_TO_DECK);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_condition(duel, thcon);
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
}

fn pdesfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_POWER_PATRON, SET_DOOMZ]) && c.is_monster(duel) && (c.is_faceup(duel) || c.is_location(duel, LOCATION_HAND))
}

fn pdestg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(pdesfilter, tp, LOCATION_MZONE | LOCATION_HAND, 0, 1, None));
    }
    let c = a.e.get_handler(duel);
    duel.set_operation_info(
        0,
        CATEGORY_DESTROY,
        Some(c.into()),
        2,
        tp,
        (LOCATION_PZONE | LOCATION_MZONE | LOCATION_HAND) as i32,
    );
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, None, 1, PLAYER_EITHER, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn pdesop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let mut g1 = duel.select_matching_card(tp, pdesfilter, tp, LOCATION_MZONE | LOCATION_HAND, 0, 1, 1, None)?;
    let Some(tc) = g1.get_first() else { return Ok(()) };
    if tc.is_on_field(duel) {
        duel.hint_selection(g1);
    }
    g1.insert(c);
    if duel.destroy(g1, REASON_EFFECT)? != 2 {
        return Ok(());
    }
    let dg = duel.get_matching_group(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    if dg.is_empty() || !duel.select_yes_no(tp, aux::stringid(ID, 3))? {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g2 = dg.select(duel, tp, 1, 1, None)?;
    if !g2.is_empty() {
        duel.hint_selection(g2);
        duel.break_effect()?;
        duel.destroy(g2, REASON_EFFECT)?;
    }
    Ok(())
}

fn jupiterspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, hc: Option<CardId>) -> bool {
    c.is_code(duel, JUPITER)
        && duel.get_location_count_from_ex(tp, tp, hc.map(CardSet::from), c) > 0
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_XYZ, tp, false, false)
}

fn mdestg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|sc, duel| jupiterspfilter(sc, duel, e, tp, Some(c)), tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(c.into()), 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn mdesop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || duel.destroy(c, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        |sc, duel| jupiterspfilter(sc, duel, e, tp, None),
        tp,
        LOCATION_EXTRA,
        0,
        1,
        1,
        None,
    )?;
    let Some(xc) = g.get_first() else { return Ok(()) };
    xc.set_material(duel, None);
    if duel.special_summon(xc, SUMMON_TYPE_XYZ, tp, tp, false, false, POS_FACEUP)? == 0 {
        return Ok(());
    }
    xc.complete_procedure(duel);
    let g = duel.get_matching_group(
        |c, duel| c.is_can_be_xyz_material(duel, Some(xc), tp, REASON_EFFECT),
        tp,
        LOCATION_HAND,
        0,
        None,
    );
    if g.is_empty() || !duel.select_yes_no(tp, aux::stringid(ID, 4))? {
        return Ok(());
    }
    let attg = g.select(duel, tp, 1, 1, None)?;
    if !attg.is_empty() {
        duel.break_effect()?;
        duel.overlay(xc, attg)?;
    }
    Ok(())
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_location(duel, LOCATION_EXTRA) && c.is_faceup(duel)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_POWER_PATRON, SET_DOOMZ]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
