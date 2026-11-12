// Daidaratant the Ooze Giant
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, PZONE};

const ID: u32 = 39943352;

pub const PENDULUM_LEVEL: u32 = 4;

// Copies in the Extra Deck, Xyz Summoned with two Level 4 monsters (Hyosube, Water Omotics); Pendulum Zone and Graveyard copies for
// the Pendulum effects, a Rank 8 Xyz Monster (Gigantic "Champion" Sargas) to attach to, and Dark Hole to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, PZONE),
    (ID, GRAVE),
    (2118022, MZONE),
    (2118022, MZONE),
    (2483611, MZONE),
    (2483611, MZONE),
    (11132674, MZONE),
    (53129443, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Pendulum::add_procedure_full(duel, c, false, None);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, ovtg);
    e1.set_operation(duel, ovop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOEXTRA);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, pltg);
    e2.set_operation(duel, plop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_condition(duel, pencon);
    e3.set_target(duel, |duel, a, chk, _| Ok(chk == 0 && duel.check_pendulum_zones(a.tp)));
    e3.set_operation(duel, penop);
    c.register_effect(duel, e3);
}

fn ovfilter(c: CardId, duel: &Duel, mc: CardId, tp: u8) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_XYZ) && mc.is_can_be_xyz_material(duel, Some(c), tp, REASON_EFFECT)
}

fn ovtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && ovfilter(chkc, duel, c, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|tc, duel| ovfilter(tc, duel, c, tp), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, |tc, duel| ovfilter(tc, duel, c, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    Ok(false)
}

fn ovop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && !tc.is_immune_to_effect(duel, a.e)
    {
        duel.overlay(tc, a.e.get_handler(duel))?;
    }
    Ok(())
}

fn pltg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.check_pendulum_zones(tp) && a.e.get_handler(duel).is_type(duel, TYPE_PENDULUM));
    }
    duel.set_possible_operation_info(0, CATEGORY_TOEXTRA, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn tefilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_PENDULUM) && !c.is_forbidden(duel)
}

fn plop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !duel.check_pendulum_zones(tp) {
        return Ok(());
    }
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e)
        && duel.move_to_field(c, tp, tp, LOCATION_PZONE, POS_FACEUP, true)?
        && duel.is_existing_matching_card(aux::necro_valley_filter(tefilter), tp, LOCATION_GRAVE, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 3))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SELECT);
        let g = duel.select_matching_card(tp, aux::necro_valley_filter(tefilter), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.break_effect()?;
            duel.send_to_extra_p(g, tp, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn pencon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_faceup(duel) && c.is_previous_location(duel, LOCATION_MZONE) && c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
}

fn penop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if !duel.check_pendulum_zones(tp) {
        return Ok(());
    }
    if c.is_relate_to_effect(duel, a.e) {
        duel.move_to_field(c, tp, tp, LOCATION_PZONE, POS_FACEUP, true)?;
    }
    Ok(())
}
