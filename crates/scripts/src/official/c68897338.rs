// The Fabled Behilmoth
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND};

const ID: u32 = 68897338;

pub const LISTED_SERIES: &[u16] = &[SET_FABLED];
// In the hand with Fabled monsters (Level 2 Cerburrel, Level 1 Lurrie) to discard for Fabled Gammajin (Level 4), and in the
// Graveyard with Fabled Kushano and Lurrie to discard from the hand.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (94292987, EXTRA),
    (94292987, EXTRA),
    (82888408, HAND),
    (97651498, HAND),
    (97651498, HAND),
    (97439806, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_HANDES + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, extrasptg);
    e1.set_operation(duel, extraspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property_full(duel, EFFECT_FLAG_DELAY, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, handspcon);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, handsptg);
    e2.set_operation(duel, handspop);
    c.register_effect(duel, e2);
}

fn discardfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_FABLED]) && c.is_monster(duel) && c.has_level(duel) && c.is_discardable_full(duel, REASON_EFFECT, None)
}

fn rescon(duel: &Duel, sg: CardSet, e: EffectId, tp: u8) -> (bool, bool) {
    let lv = sg.get_sum(duel, |c, duel| i64::from(c.get_original_level(duel))) as u32 + e.get_handler(duel).get_original_level(duel);
    (
        duel.is_existing_matching_card(|c, duel| extraspfilter(c, duel, e, tp, lv), tp, LOCATION_EXTRA, 0, 1, None),
        false,
    )
}

fn extraspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, lv: u32) -> bool {
    c.is_set_card(duel, &[SET_FABLED])
        && c.is_type(duel, TYPE_SYNCHRO)
        && c.is_level(duel, &[lv])
        && duel.get_location_count_from_ex(tp, tp, None, c) > 0
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_SYNCHRO, tp, false, false)
}

fn extrasptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let pg = aux::get_must_be_material_group(duel, tp, Some(CardSet::default()), tp, None, None, REASON_SYNCHRO);
        if !pg.is_empty() {
            return Ok(false);
        }
        let c = e.get_handler(duel);
        let g = duel.get_matching_group(discardfilter, tp, LOCATION_HAND, 0, Some(c.into()));
        return Ok(discardfilter(c, duel)
            && !g.is_empty()
            && aux::select_unselect_group_check(duel, g, 1, g.len(), &|duel, sg, _, _| rescon(duel, sg, e, tp)));
    }
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, tp, 2);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn extraspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let g = duel.get_matching_group(discardfilter, tp, LOCATION_HAND, 0, Some(c.into()));
    if g.is_empty() {
        return Ok(());
    }
    let mut rg = aux::select_unselect_group(duel, g, 1, g.len(), &|duel, sg, _, _| rescon(duel, sg, e, tp), tp, HINTMSG_DISCARD)?;
    if rg.is_empty() {
        return Ok(());
    }
    rg.insert(c);
    if duel.send_to_grave(rg, REASON_DISCARD | REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let pg = aux::get_must_be_material_group(duel, tp, Some(CardSet::default()), tp, None, None, REASON_SYNCHRO);
    if !pg.is_empty() {
        return Ok(());
    }
    let lv = rg.get_sum(duel, |c, duel| i64::from(c.get_original_level(duel))) as u32;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let sg = duel.select_matching_card(tp, |c, duel| extraspfilter(c, duel, e, tp, lv), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if let Some(sc) = sg.first()
        && duel.special_summon(sc, SUMMON_TYPE_SYNCHRO, tp, tp, false, false, POS_FACEUP)? > 0
    {
        sc.complete_procedure(duel);
    }
    Ok(())
}

fn handspconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_previous_controler(duel, tp) && c.is_previous_location(duel, LOCATION_HAND)
}

fn handspcon(duel: &Duel, a: &Args) -> bool {
    let eg = a.eg.unwrap_or_default();
    !eg.is_contains(a.e.get_handler(duel)) && eg.is_exists(duel, |c, duel| handspconfilter(c, duel, a.tp), 1, None)
}

fn handspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_FABLED]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn handsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| handspfilter(c, duel, a.e, a.tp), LOCATION_HAND)
}

fn handspop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| handspfilter(c, duel, a.e, a.tp), LOCATION_HAND, POS_FACEUP)
}
