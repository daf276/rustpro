// Ghostrick Shot
use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE};

const ID: u32 = 69809989;

pub const LISTED_SERIES: &[u16] = &[SET_GHOSTRICK];
// Another copy in the GY to attach, to Ghostrick Alucard. No Ghostrick monster outside the Extra Deck is ported yet, so the
// first effect has no target.
pub const SUPPORT: &[(u32, u16)] = &[(ID, GRAVE), (75367227, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_POSITION);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_LEAVE_GRAVE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, mattg);
    e2.set_operation(duel, matop);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_GHOSTRICK]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let locations = LOCATION_HAND | LOCATION_GRAVE;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, locations, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, locations as i32);
    Ok(false)
}

fn posfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GHOSTRICK]) && c.is_facedown(duel) && c.is_defense_pos(duel) && c.is_can_change_position(duel)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, 1, None)?;
    if !g.is_empty()
        && duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)? > 0
        && duel.is_existing_matching_card(posfilter, tp, LOCATION_MZONE, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
        let pg = duel.select_matching_card(tp, posfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
        if !pg.is_empty() {
            duel.break_effect()?;
            duel.change_position(pg, POS_FACEUP_ATTACK)?;
        }
    }
    Ok(())
}

fn attachfilter(c: CardId, duel: &Duel, tp: u8, xyzc: CardId) -> bool {
    c.is_set_card(duel, &[SET_GHOSTRICK]) && c.is_can_be_xyz_material(duel, Some(xyzc), tp, REASON_EFFECT)
}

fn xyzfilter(c: CardId, duel: &Duel, tp: u8, exc: CardId) -> bool {
    c.is_faceup(duel)
        && c.is_type(duel, TYPE_XYZ)
        && c.is_set_card(duel, &[SET_GHOSTRICK])
        && duel.is_existing_matching_card(|g, duel| attachfilter(g, duel, tp, c), tp, LOCATION_GRAVE, 0, 1, Some(exc.into()))
}

fn mattg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    let filter = |g: CardId, duel: &Duel| xyzfilter(g, duel, tp, c);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, filter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, None, 1, tp, 0);
    Ok(false)
}

fn matop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel) && tc.is_type(duel, TYPE_XYZ) && !tc.is_immune_to_effect(duel, e) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_XMATERIAL);
        let g = duel.select_matching_card(tp, |c, duel| attachfilter(c, duel, tp, tc), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.overlay(tc, g)?;
        }
    }
    Ok(())
}
