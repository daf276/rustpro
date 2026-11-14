// Goblin Biker Grand Pileup
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, SZONE};

const ID: u32 = 90946420;

pub const LISTED_SERIES: &[u16] = &[SET_GOBLIN];
// Goblin Biker Big Gabonga on the field as the Xyz material, Troika Griare in the Extra Deck to summon, and a copy in the
// Graveyard to attach a card to Big Gabonga. No ported Goblin Xyz Monster has a rum_limit, so that check is left out.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (ID, GRAVE),
    (34001672, MZONE),
    (34001672, MZONE),
    (44241999, EXTRA),
    (44241999, EXTRA),
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
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, xyzsumtg);
    e1.set_operation(duel, xyzsumop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, mattg);
    e2.set_operation(duel, matop);
    c.register_effect(duel, e2);
}

fn must_be_material_group(duel: &Duel, c: CardId, tp: u8) -> CardSet {
    aux::get_must_be_material_group(duel, tp, Some(c.into()), tp, None, None, REASON_XYZ)
}

fn xyzmatfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    let pg = must_be_material_group(duel, c, tp);
    (pg.is_empty() || (pg.len() == 1 && pg.contains(c)))
        && c.is_faceup(duel)
        && c.is_type(duel, TYPE_XYZ)
        && duel.is_existing_matching_card(|sc, duel| spfilter(sc, duel, e, tp, c), tp, LOCATION_EXTRA, 0, 1, None)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: CardId) -> bool {
    c.is_set_card(duel, &[SET_GOBLIN])
        && c.is_type(duel, TYPE_XYZ)
        && duel.get_location_count_from_ex(tp, tp, Some(mc.into()), c) > 0
        && mc.is_can_be_xyz_material(duel, Some(c), duel.get_reason_player(), REASON_XYZ | REASON_MATERIAL)
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_XYZ, tp, false, false)
}

fn xyzsumtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let filter = |c, duel: &Duel| xyzmatfilter(c, duel, e, tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, filter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn xyzsumop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    let pg = must_be_material_group(duel, tc, tp);
    if tc.is_facedown(duel)
        || !tc.is_relate_to_effect(duel, e)
        || tc.is_controler(duel, 1 - tp)
        || tc.is_immune_to_effect(duel, e)
        || pg.len() > 1
        || (pg.len() == 1 && !pg.contains(tc))
    {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |sc, duel| spfilter(sc, duel, e, tp, tc), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first() {
        sc.set_material(duel, Some(tc.into()));
        duel.overlay(sc, tc)?;
        duel.special_summon(sc, SUMMON_TYPE_XYZ, tp, tp, false, false, POS_FACEUP)?;
        sc.complete_procedure(duel);
        let c = e.get_handler(duel);
        if c.is_relate_to_effect(duel, e) {
            c.cancel_to_grave(duel);
            duel.overlay(sc, c)?;
        }
    }
    Ok(())
}

fn xyzfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_XYZ) && c.is_set_card(duel, &[SET_GOBLIN])
}

fn atchfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_controler(duel, tp) || c.is_able_to_change_controler(duel)
}

fn mattg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && xyzfilter(chkc, duel));
    }
    let atch = |c, duel: &Duel| atchfilter(c, duel, tp);
    if chk == 0 {
        return Ok(duel.is_existing_target(xyzfilter, tp, LOCATION_MZONE, 0, 1, None)
            && duel.is_existing_target(atch, tp, LOCATION_GRAVE, LOCATION_GRAVE, 1, Some(e.get_handler(duel).into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let xyzc = duel.select_target(tp, xyzfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    e.set_label_object_group(duel, xyzc);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATTACH);
    let tc = duel.select_target(tp, atch, tp, LOCATION_GRAVE, LOCATION_GRAVE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, tc, 1, 0, 0);
    Ok(false)
}

fn matop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let g = duel.get_target_cards(e).unwrap_or_default();
    let mut cards = g.iter();
    let (Some(first), Some(second), None) = (cards.next(), cards.next(), cards.next()) else {
        return Ok(());
    };
    let label = e.get_label_object_group(duel).and_then(|lg| lg.get_first());
    let (tc, xyzc) = if Some(first) == label { (second, first) } else { (first, second) };
    duel.overlay(xyzc, tc)
}
