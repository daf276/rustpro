// Number 53: Heart-eartH
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};
use engine::constants::MSG_SELECT_EFFECTYN;

const ID: u32 = 23998625;
pub const XYZ_NUMBER: u32 = 53;
const HEART_EARTH_DRAGON: u32 = 97403510;

pub const LISTED_NAMES: &[u32] = &[HEART_EARTH_DRAGON];

// In the Extra Deck, Xyz Summoned with the only three monsters (Ocubeam, Level 5), or on the field with Cyber Falcon attached by Xyz
// Import, with the opponent's monsters to battle it and the opponent's Smashing Ground and Fissure to destroy it. Number 92: Heart-eartH
// Dragon is not ported, so the Special Summon has no scenario.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (ID, MZONE),
    (86088138, MZONE),
    (86088138, MZONE),
    (86088138, MZONE),
    (14602126, HAND),
    (14602126, HAND),
    (30655537, MZONE | OPPONENT),
    (30655537, MZONE | OPPONENT),
    (30655537, MZONE | OPPONENT),
    (97169186, HAND | OPPONENT),
    (66788016, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Message(MSG_SELECT_EFFECTYN),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(5);
    const CT: usize = 3;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_BE_BATTLE_TARGET);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, atktg);
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_DESTROY_REPLACE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target(duel, reptg);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_condition(duel, spcon);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn atktg(duel: &mut Duel, _: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let attacker = duel.get_attacker().map(CardSet::from).unwrap_or_default();
    duel.set_target_card(attacker);
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if let Some(tc) = duel.get_first_target()
        && c.is_relate_to_effect(duel, e)
        && c.is_faceup(duel)
        && tc.is_relate_to_effect(duel, e)
        && tc.can_attack(duel)
        && !tc.is_status(duel, STATUS_ATTACK_CANCELED)
    {
        let atk = tc.get_base_attack(duel);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, atk);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(!c.is_reason(duel, REASON_REPLACE) && c.check_remove_overlay_card(duel, tp, 1, REASON_EFFECT));
    }
    if !duel.select_effect_yes_no_full(tp, c, 96)? {
        return Ok(false);
    }
    c.remove_overlay_card(duel, tp, 1, 1, REASON_EFFECT)?;
    Ok(true)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_location(duel, LOCATION_MZONE)
        && c.is_reason(duel, REASON_EFFECT)
        && c.get_overlay_count(duel) == 0
        && c.is_location(duel, LOCATION_GRAVE)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, rp: u8) -> bool {
    c.is_code(duel, HEART_EARTH_DRAGON)
        && duel.get_location_count_from_ex(tp, rp, None, c) > 0
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_XYZ, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp, rp) = (a.e, a.tp, a.rp);
    if chk == 0 {
        let pg = aux::get_must_be_material_group(duel, tp, Some(CardSet::default()), tp, None, None, REASON_XYZ);
        return Ok(pg.is_empty() && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp, rp), tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(e.get_handler(duel).into()), 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp, rp) = (a.e, a.tp, a.rp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let pg = aux::get_must_be_material_group(duel, tp, Some(CardSet::default()), tp, None, None, REASON_XYZ);
    if !pg.is_empty() {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp, rp), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        duel.overlay(tc, c)?;
        duel.special_summon(tc, SUMMON_TYPE_XYZ, tp, tp, false, false, POS_FACEUP)?;
        tc.complete_procedure(duel);
    }
    Ok(())
}
