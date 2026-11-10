// Shamanite Shamanknight
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT, REMOVED};

const ID: u32 = 16237004;

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(3);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;
}

// Baby Dragon (Level 3) to Xyz Summon it, a banished Trap Hole to attach; on the field with Dragon Rebellion and Sangan taken
// as materials by Xyz Import to detach, and Kozaky (Level 1, DARK) in the Graveyard to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (88819587, MZONE),
    (88819587, MZONE),
    (88819587, MZONE),
    (88819587, MZONE),
    (4206964, REMOVED),
    (4206964, REMOVED),
    (99171160, GRAVE),
    (99171160, GRAVE),
    (14602126, HAND),
    (14602126, HAND),
    (16195942, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1a = duel.create_effect(c);
    e1a.set_description(duel, aux::stringid(ID, 0));
    e1a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1a.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e1a.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1a.set_count_limit(duel, 1, (ID, 0));
    e1a.set_condition(duel, |duel, a| a.e.get_handler(duel).is_xyz_summoned(duel));
    e1a.set_target(duel, attachtg);
    e1a.set_operation(duel, attachop);
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1b.set_code(duel, EVENT_REMOVE);
    e1b.set_range(duel, LOCATION_MZONE);
    e1b.set_condition(duel, |duel, a| {
        a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| trapfilter(c, duel, a.tp), 1, None))
    });
    c.register_effect(duel, e1b);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, spcost);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn attachfilter(c: CardId, duel: &Duel, xyzc: CardId, tp: u8) -> bool {
    c.is_trap(duel) && c.is_faceup(duel) && c.is_can_be_xyz_material(duel, Some(xyzc), tp, REASON_EFFECT)
}

fn attachtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_REMOVED) && attachfilter(chkc, duel, c, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|tc, duel| attachfilter(tc, duel, c, tp), tp, LOCATION_REMOVED, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATTACH);
    duel.select_target(tp, |tc, duel| attachfilter(tc, duel, c, tp), tp, LOCATION_REMOVED, 0, 1, 1, None)?;
    Ok(false)
}

fn attachop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && c.is_relate_to_effect(duel, e)
        && tc.is_relate_to_effect(duel, e)
        && tc.is_can_be_xyz_material(duel, Some(c), tp, REASON_EFFECT)
    {
        duel.overlay(c, tc)?;
    }
    Ok(())
}

fn trapfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_trap(duel) && c.is_faceup(duel) && c.is_owner(duel, tp)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, level: Option<u32>) -> bool {
    c.is_attribute(duel, ATTRIBUTE_DARK)
        && c.has_level(duel)
        && level.is_none_or(|level| c.is_level(duel, &[level]))
        && c.is_faceup(duel)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let choices = duel
        .get_target_group(
            |c, duel| spfilter(c, duel, e, tp, None),
            tp,
            LOCATION_GRAVE | LOCATION_REMOVED,
            0,
            None,
        )
        .get_class(duel, |c, duel| i64::from(c.get_level(duel)));
    Cost::detach_choice_from_self(duel, a, chk, &choices)
}

fn detached_count(duel: &Duel, e: EffectId) -> u32 {
    e.get_chain_data_group(duel, COST_DETACHED_MATERIALS).unwrap_or_default().len() as u32
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        let level = Some(detached_count(duel, e));
        return Ok(chkc.is_controler(duel, tp)
            && chkc.is_location(duel, LOCATION_GRAVE | LOCATION_REMOVED)
            && spfilter(chkc, duel, e, tp, level));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(
                |c, duel| spfilter(c, duel, e, tp, None),
                tp,
                LOCATION_GRAVE | LOCATION_REMOVED,
                0,
                1,
                None,
            ));
    }
    let level = Some(detached_count(duel, e));
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(
        tp,
        |c, duel| spfilter(c, duel, e, tp, level),
        tp,
        LOCATION_GRAVE | LOCATION_REMOVED,
        0,
        1,
        1,
        None,
    )?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
    {
        duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
