// Exosister Elis
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 16474916;

pub const LISTED_SERIES: &[u16] = &[SET_EXOSISTER];
pub const LISTED_NAMES: &[u32] = &[43863925];

// On the field and in the hand with Exosister Stella on the field, and the opponent's Monster Reborn to take a monster out of a
// Graveyard, for Exosister Karmael in the Extra Deck to be Xyz Summoned.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (43863925, MZONE),
    (43863925, HAND),
    (77675029, EXTRA),
    (77675029, EXTRA),
    (83764718, HAND | OPPONENT),
    (83764718, HAND | OPPONENT),
    (15023985, GRAVE),
    (15023985, GRAVE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_RECOVER);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, hspcon);
    e1.set_target(duel, stock::special_summon_self_target_player_0);
    e1.set_operation(duel, hspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_LEAVE_GRAVE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |_, a| a.rp == 1 - a.tp);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn hspcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(
        aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_EXOSISTER])),
        a.tp,
        LOCATION_MZONE,
        0,
        1,
        None,
    )
}

fn hspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e)
        && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0
        && duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_code(duel, 43863925)),
            tp,
            LOCATION_ONFIELD,
            0,
            1,
            None,
        )
    {
        duel.break_effect()?;
        duel.recover(tp, 800, REASON_EFFECT)?;
    }
    Ok(())
}

pub fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: CardId) -> bool {
    c.is_type_full(duel, TYPE_XYZ, Some(c), SUMMON_TYPE_XYZ as u64, tp)
        && c.is_set_card(duel, &[SET_EXOSISTER])
        && mc.is_can_be_xyz_material(duel, Some(c), tp, REASON_XYZ | REASON_MATERIAL)
        && duel.get_location_count_from_ex(tp, tp, Some(mc.into()), c) > 0
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_XYZ, tp, false, false)
}

pub fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let c = e.get_handler(duel);
        let pg = aux::get_must_be_material_group(duel, tp, Some(c.into()), tp, None, None, REASON_XYZ);
        return Ok((pg.is_empty() || (pg.len() == 1 && pg.contains(c)))
            && duel.is_existing_matching_card(|sc, duel| spfilter(sc, duel, e, tp, c), tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

pub fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_facedown(duel) || !c.is_relate_to_effect(duel, e) || c.is_controler(duel, 1 - tp) || c.is_immune_to_effect(duel, e) {
        return Ok(());
    }
    let pg = aux::get_must_be_material_group(duel, tp, Some(c.into()), tp, None, None, REASON_XYZ);
    if pg.len() > 1 || (pg.len() == 1 && !pg.contains(c)) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |sc, duel| spfilter(sc, duel, e, tp, c), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first() {
        sc.set_material(duel, Some(c.into()));
        duel.overlay(sc, c)?;
        if duel.special_summon(sc, SUMMON_TYPE_XYZ, tp, tp, false, false, POS_FACEUP)? > 0 {
            sc.complete_procedure(duel);
        }
    }
    Ok(())
}
