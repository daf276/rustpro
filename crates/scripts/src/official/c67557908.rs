// Number 4: Stealth Kragen
use super::c94942656::{faceup_water, is_water, recordop};
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 67557908;

pub const LISTED_SERIES: &[u16] = &[SET_NUMBER];
pub const XYZ_NUMBER: u32 = 4;
const SPAWN: u32 = 94942656;

pub const LISTED_NAMES: &[u32] = &[SPAWN];

// In the Extra Deck with Stealth Kragen Spawn and WATER monsters to Xyz Summon, WATER monsters in the Graveyard to attach,
// and Dark Hole and an opponent's copy to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (SPAWN, EXTRA),
    (2118022, MZONE),
    (2483611, MZONE),
    (4042268, GRAVE),
    (5628232, GRAVE),
    (53129443, HAND),
    (53129443, HAND),
    (ID, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummonedCode(SPAWN),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_attribute_full(duel, ATTRIBUTE_WATER, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_CHANGE_ATTRIBUTE);
    e1.set_value(duel, ATTRIBUTE_WATER as i32);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DESTROY + CATEGORY_DAMAGE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_MAIN_END);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, |duel, _| duel.is_main_phase());
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_TRIGGER_O + EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_condition(duel, spcon);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e4.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e4.set_code(duel, EVENT_LEAVE_FIELD_P);
    e4.set_operation(duel, recordop);
    e4.set_label_object(duel, Some(e3));
    c.register_effect(duel, e4);
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(faceup_water, tp, 0, LOCATION_MZONE, 1, None));
    }
    let g = duel.get_matching_group(faceup_water, tp, 0, LOCATION_MZONE, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_matching_card(tp, faceup_water, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        let atk = (f64::from(tc.get_attack(duel)) / 2.0).round().max(0.0) as u32;
        duel.hint_selection(g);
        if duel.destroy(tc, REASON_EFFECT)? > 0 {
            duel.damage(1 - tp, atk, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_location(duel, LOCATION_MZONE) && c.is_xyz_summoned(duel)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, rp: u8) -> bool {
    c.is_code(duel, SPAWN)
        && duel.get_location_count_from_ex(tp, rp, None, c) > 0
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp, rp) = (a.e, a.tp, a.rp);
    if chk == 0 {
        let materials = e.get_label_object_group(duel).unwrap_or_default();
        return Ok(
            !materials.is_empty() && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp, rp), tp, LOCATION_EXTRA, 0, 1, None)
        );
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp, rp) = (a.e, a.tp, a.rp);
    let mut ct = e.get_label_object_group(duel).unwrap_or_default().len() as i32;
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ct = 1;
    }
    ct = ct.min(aux::check_summon_gate(duel, tp).unwrap_or(ct));
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let mut g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp, rp), tp, LOCATION_EXTRA, 0, 1, ct as u8, None)?;
    if g.is_empty() {
        return Ok(());
    }
    let count = duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    if count == 0 {
        return Ok(());
    }
    let sg = duel.get_matching_group(aux::necro_valley_filter(is_water), tp, LOCATION_GRAVE, 0, None);
    if !sg.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        duel.break_effect()?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_XMATERIAL);
        for oc in sg.select(duel, tp, 1, count as u8, None)? {
            let on_field = |c: CardId, duel: &Duel| c.is_location(duel, LOCATION_MZONE);
            let Some(tc) = g.filter_select(duel, tp, on_field, 1, 1, None)?.get_first() else {
                break;
            };
            duel.overlay(tc, oc)?;
            g.remove(tc);
        }
    }
    Ok(())
}
