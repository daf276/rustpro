// Number 27: Dreadnought Dreadnoid
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};
use engine::constants::MSG_SELECT_EFFECTYN;

const ID: u32 = 8387138;
pub const XYZ_NUMBER: u32 = 27;

// In the Extra Deck, Xyz Summoned with Level 4 Normal Monsters, with a Rank 10 Machine (Number 81) to Special Summon, copies in the
// Main Monster Zone and weak Sangans of the opponent's to battle, and the opponent's Raigeki to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (ID, MZONE),
    (49032236, EXTRA),
    (2118022, MZONE),
    (2483611, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Message(MSG_SELECT_EFFECTYN)];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_BATTLE_DESTROYING);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_operation(duel, regop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_PHASE | PHASE_BATTLE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).get_flag_effect(duel, ID) != 0);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_code(duel, EFFECT_DESTROY_REPLACE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target(duel, reptg);
    c.register_effect(duel, e3);
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    a.e.get_handler(duel)
        .register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
    Ok(())
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    let handler = e.get_handler(duel);
    c.is_rank_above(duel, 10)
        && c.is_race(duel, RACE_MACHINE)
        && handler.is_can_be_xyz_material(duel, Some(c), tp, REASON_XYZ | REASON_MATERIAL)
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_XYZ, tp, false, false)
        && duel.get_location_count_from_ex(tp, tp, Some(handler.into()), c) > 0
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(
        duel,
        a,
        chk,
        |c, duel| filter(c, duel, a.e, a.tp),
        LOCATION_EXTRA,
        CATEGORY_SPECIAL_SUMMON,
    )
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c, tp) = (a.e, a.e.get_handler(duel), a.tp);
    let g = duel.get_matching_group(|sc, duel| filter(sc, duel, e, tp), tp, LOCATION_EXTRA, 0, None);
    if g.is_empty()
        || c.is_facedown(duel)
        || !c.is_relate_to_effect(duel, e)
        || c.is_controler(duel, 1 - tp)
        || c.is_immune_to_effect(duel, e)
    {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = g.select(duel, tp, 1, 1, None)?;
    if let Some(sc) = g.get_first() {
        sc.set_material(duel, Some(c.into()));
        duel.overlay(sc, c)?;
        duel.special_summon(sc, SUMMON_TYPE_XYZ, tp, tp, false, false, POS_FACEUP)?;
        sc.complete_procedure(duel);
    }
    Ok(())
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(!c.is_reason(duel, REASON_REPLACE) && c.check_remove_overlay_card(duel, tp, 1, REASON_EFFECT));
    }
    if duel.select_effect_yes_no_full(tp, c, 96)? {
        c.remove_overlay_card(duel, tp, 1, 1, REASON_EFFECT)?;
        return Ok(true);
    }
    Ok(false)
}
