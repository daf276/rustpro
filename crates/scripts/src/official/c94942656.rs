// Stealth Kragen Spawn
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 94942656;
const KRAGEN: u32 = 67557908;

// Number 4: Stealth Kragen in the Extra Deck to Special Summon this card, and Stealth Kragen monsters and WATER
// monsters in the Graveyard for its Destroyed effect.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (KRAGEN, EXTRA),
    (KRAGEN, GRAVE),
    (2118022, MZONE),
    (2483611, MZONE),
    (4042268, GRAVE),
    (5628232, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::SpecialSummonedCode(KRAGEN)];

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
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_MAIN_END);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_operation(duel, regop);
    c.register_effect(duel, e2);
}

pub fn faceup_water(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_attribute(duel, ATTRIBUTE_WATER)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(faceup_water, a.tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, None, 1, 1 - a.tp, LOCATION_MZONE as i32);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.hint(HINT_SELECTMSG, a.tp, HINTMSG_DESTROY);
    let g = duel.select_matching_card(a.tp, faceup_water, a.tp, 0, LOCATION_MZONE, 1, 1, None)?;
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(re) = a.re else { return Ok(()) };
    let rc = re.get_handler(duel);
    if re.is_monster_effect(duel) && rc.is_set_card(duel, &[SET_NUMBER]) && rc.is_type(duel, TYPE_XYZ) {
        let reset = RESET_EVENT | (RESETS_STANDARD & !(RESET_TOGRAVE | RESET_LEAVE | RESET_REMOVE));
        let e3 = duel.create_effect(c);
        e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
        e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
        e3.set_code(duel, EVENT_DESTROYED);
        e3.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
        e3.set_reset(duel, reset);
        e3.set_condition(duel, |duel, a| a.e.get_handler(duel).is_previous_location(duel, LOCATION_MZONE));
        e3.set_target(duel, sptg);
        e3.set_operation(duel, spop);
        c.register_effect(duel, e3);
        let e4 = duel.create_effect(c);
        e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
        e4.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e4.set_code(duel, EVENT_LEAVE_FIELD_P);
        e4.set_operation(duel, recordop);
        e4.set_reset(duel, reset);
        e4.set_label_object(duel, Some(e3));
        c.register_effect(duel, e4);
    }
    Ok(())
}

// Records the materials before this card leaves the field.
pub fn recordop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = a.e.get_handler(duel).get_overlay_group(duel);
    if let Some(e3) = a.e.get_label_object(duel) {
        e3.set_label_object_group(duel, Some(g));
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_STEALTH_KRAGEN]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let materials = e.get_label_object_group(duel).unwrap_or_default();
        let c = e.get_handler(duel);
        return Ok(!materials.is_empty()
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|tc, duel| spfilter(tc, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, Some(c.into())));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let mut ct = duel
        .get_location_count(tp, LOCATION_MZONE)
        .min(e.get_label_object_group(duel).unwrap_or_default().len() as i32);
    if ct < 1 {
        return Ok(());
    }
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ct = 1;
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|tc, duel| spfilter(tc, duel, e, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_GRAVE, 0, 1, ct as u8, Some(c.into()))?;
    if !g.is_empty() && duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        let og = duel.get_operated_group();
        let mg = duel.get_matching_group(aux::necro_valley_filter(is_water), tp, LOCATION_GRAVE, 0, None);
        if !og.is_empty() && !mg.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
            duel.break_effect()?;
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_XMATERIAL);
            let xg = mg.select(duel, tp, 1, og.len() as u8, None)?;
            for xc in xg {
                duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
                if let Some(tc) = og.select(duel, tp, 1, 1, None)?.get_first() {
                    duel.overlay(tc, xc)?;
                }
            }
        }
    }
    Ok(())
}

pub fn is_water(c: CardId, duel: &Duel) -> bool {
    c.is_attribute(duel, ATTRIBUTE_WATER)
}
