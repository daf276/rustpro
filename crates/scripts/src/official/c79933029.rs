// Purrelyly
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 79933029;

pub const LISTED_SERIES: &[u16] = &[SET_PURRELY];

// Purrely in the Deck to search, Purrely Happy Memory in the GY to attach to Epurrely Happiness, which lists it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (25550531, DECK),
    (25550531, DECK),
    (82105704, GRAVE),
    (82105704, GRAVE),
    (52645235, EXTRA),
    (52645235, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_LEAVE_GRAVE);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_PURRELY]) && !c.is_quick_play_spell(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn qpfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: CardId) -> bool {
    c.is_set_card(duel, &[SET_PURRELY])
        && c.is_quick_play_spell(duel)
        && duel.is_existing_matching_card(|xc, duel| spfilter(xc, duel, e, tp, mc, c), tp, LOCATION_EXTRA, 0, 1, None)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: CardId, sc: CardId) -> bool {
    c.is_type_full(duel, TYPE_XYZ, Some(c), SUMMON_TYPE_XYZ as u64, tp)
        && c.lists_code(duel, sc.get_code(duel))
        && mc.is_can_be_xyz_material(duel, Some(c), tp, REASON_XYZ | REASON_MATERIAL)
        && sc.is_can_be_xyz_material(duel, Some(c), tp, REASON_EFFECT)
        && duel.get_location_count_from_ex(tp, tp, Some(mc.into()), c) > 0
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_XYZ, tp, false, false)
}

pub fn must_material_ok(duel: &Duel, tp: u8, c: CardId) -> bool {
    let pg = aux::get_must_be_material_group(duel, tp, Some(c.into()), tp, None, None, REASON_XYZ);
    pg.is_empty() || (pg.len() == 1 && pg.contains(c))
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(must_material_ok(duel, tp, c)
            && duel.is_existing_target(|qc, duel| qpfilter(qc, duel, e, tp, c), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g = duel.select_target(tp, |qc, duel| qpfilter(qc, duel, e, tp, c), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, g, 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_facedown(duel) || !c.is_relate_to_effect(duel, e) || c.is_controler(duel, 1 - tp) || c.is_immune_to_effect(duel, e) {
        return Ok(());
    }
    if !must_material_ok(duel, tp, c) {
        return Ok(());
    }
    let Some(tc) = duel.get_first_target() else {
        return Ok(());
    };
    if !tc.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let Some(xc) = duel
        .select_matching_card(tp, |xc, duel| spfilter(xc, duel, e, tp, c, tc), tp, LOCATION_EXTRA, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    xc.set_material(duel, Some(c.into()));
    duel.overlay(xc, c)?;
    if duel.special_summon(xc, SUMMON_TYPE_XYZ, tp, tp, false, false, POS_FACEUP)? > 0 {
        xc.complete_procedure(duel);
        duel.overlay(xc, tc)?;
    }
    Ok(())
}
