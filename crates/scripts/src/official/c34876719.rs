// N.As.H. Knight
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE};

const ID: u32 = 34876719;

pub const LISTED_SERIES: &[u16] = &[SET_NUMBER];
// On the field with Xyz materials, with Number 101, 104 and 106 in the Extra Deck to attach.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, EXTRA), (48739166, EXTRA), (2061963, EXTRA), (63746411, EXTRA)];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(5);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_condition(duel, indcon);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, _| duel.is_main_phase());
    e2.set_cost(duel, Cost::detach_from_self::<2, 2>);
    e2.set_target(duel, atttg);
    e2.set_operation(duel, attop);
    c.register_effect(duel, e2);
}

fn indcon(duel: &Duel, _: &Args) -> bool {
    duel.is_existing_matching_card(
        |c, duel| c.is_faceup(duel) && c.is_set_card(duel, &[SET_NUMBER]),
        0,
        LOCATION_MZONE,
        LOCATION_MZONE,
        1,
        None,
    )
}

pub(crate) fn is_number_10x(c: CardId, duel: &Duel) -> bool {
    c.xyz_number(duel).is_some_and(|no| (101..=107).contains(&no)) && c.is_set_card(duel, &[SET_NUMBER]) && c.is_type(duel, TYPE_XYZ)
}

fn attfilter1(c: CardId, duel: &Duel, e: EffectId) -> bool {
    is_number_10x(c, duel) && !c.is_immune_to_effect(duel, e)
}

fn atttg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(
            duel.is_existing_matching_card(|c, duel| attfilter1(c, duel, e), tp, LOCATION_EXTRA, 0, 1, None)
                && e.get_handler(duel).is_type(duel, TYPE_XYZ),
        );
    }
    Ok(false)
}

fn attfilter2(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_faceup(duel) && !c.is_type(duel, TYPE_TOKEN) && !c.is_immune_to_effect(duel, e)
}

fn attop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_XMATERIAL);
    let exg = duel.select_matching_card(tp, |c, duel| attfilter1(c, duel, e), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if exg.is_empty() {
        return Ok(());
    }
    duel.overlay(c, exg)?;
    let g = duel.get_matching_group(|c, duel| attfilter2(c, duel, e), tp, LOCATION_MZONE, LOCATION_MZONE, Some(c.into()));
    if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 1))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_XMATERIAL);
        let Some(tc) = g.select(duel, tp, 1, 1, None)?.get_first() else {
            return Ok(());
        };
        duel.hint_selection_full(tc, true);
        duel.break_effect()?;
        duel.overlay_full(c, tc, true)?;
    }
    Ok(())
}
