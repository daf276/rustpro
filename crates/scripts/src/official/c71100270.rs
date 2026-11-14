// Goblin's Crazy Beast
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT, SZONE};
use engine::constants::MSG_SELECT_EFFECTYN;

const ID: u32 = 71100270;

pub const LISTED_SERIES: &[u16] = &[SET_GOBLIN];
// The destruction replacement is no chain link and moves no card; its prompt is the MSG_SELECT_EFFECTYN.
// On the field with materials attached by Xyz Import from the opponent's Sangan, and Set Spells and Traps to attach; a copy in the
// Graveyard to summon with Goblin Bikers Gone Bonkers to attach, and in the Extra Deck with two Frostosaurus, Level 6.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, GRAVE),
    (ID, EXTRA),
    (6631034, MZONE),
    (6631034, MZONE),
    (46014517, GRAVE),
    (14602126, HAND),
    (14602126, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (88279736, SZONE),
    (88279736, SZONE | OPPONENT),
    (12580477, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::Message(MSG_SELECT_EFFECTYN),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(6);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, dttg);
    e1.set_operation(duel, dtop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EFFECT_DESTROY_REPLACE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target(duel, reptg);
    e2.set_operation(duel, |duel, a| {
        a.e.get_handler(duel).remove_overlay_card(duel, a.tp, 1, 1, REASON_EFFECT)?;
        Ok(())
    });
    e2.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(repfilter(c, duel, e.get_handler_player(duel)))),
    );
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_DUEL);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn dttg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    let stfilter = |tc: CardId, duel: &Duel| tc.is_spell_trap(duel) && tc.is_can_be_xyz_material(duel, Some(c), tp, REASON_EFFECT);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && stfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.check_remove_overlay_card(tp, 1, 1, 2, REASON_EFFECT)
            && duel.is_existing_target(stfilter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATTACH);
    duel.select_target(tp, stfilter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    Ok(false)
}

fn dtop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.remove_overlay_card(tp, 1, 1, 2, 2, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let c = e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && c.is_relate_to_effect(duel, e)
        && tc.is_relate_to_effect(duel, e)
        && !tc.is_immune_to_effect(duel, e)
        && tc.is_can_be_xyz_material(duel, Some(c), tp, REASON_EFFECT)
    {
        duel.overlay(c, tc)?;
    }
    Ok(())
}

fn repfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_location(duel, LOCATION_MZONE)
        && c.is_controler(duel, tp)
        && c.is_reason(duel, REASON_EFFECT)
        && !c.is_reason(duel, REASON_REPLACE)
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(a
            .eg
            .unwrap_or_default()
            .is_exists(duel, |tc, duel| repfilter(tc, duel, tp), 1, None)
            && c.check_remove_overlay_card(duel, tp, 1, REASON_EFFECT));
    }
    duel.select_effect_yes_no_full(tp, c, 96)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    duel.set_possible_operation_info(0, CATEGORY_LEAVE_GRAVE, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn gfilter(c: CardId, duel: &Duel, xc: CardId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_GOBLIN]) && c.is_monster(duel) && c.is_can_be_xyz_material(duel, Some(xc), tp, REASON_EFFECT)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e)
        && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0
        && duel.is_existing_matching_card(|tc, duel| gfilter(tc, duel, c, tp), tp, LOCATION_GRAVE, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATTACH);
        let g = duel.select_matching_card(tp, |tc, duel| gfilter(tc, duel, c, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
        if g.is_empty() {
            return Ok(());
        }
        duel.hint_selection_full(g, true);
        duel.break_effect()?;
        duel.overlay(c, g)?;
    }
    Ok(())
}
