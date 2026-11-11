// Ryzeal Detonator
use engine::constants::MSG_SELECT_EFFECTYN;

use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 34909328;

// In the Extra Deck, Xyz Summoned with Ryzeal monsters on the field and others in the Graveyard to attach, and the opponent's
// Raigeki and monsters to respond and destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (8633261, MZONE),
    (35844557, MZONE),
    (72238166, MZONE),
    (34022970, GRAVE),
    (34022970, GRAVE),
    (12580477, HAND | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (14558127, MZONE | OPPONENT),
];
// The destruction replacement is no chain link; its prompt is the MSG_SELECT_EFFECTYN, counted for any card.
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::Message(MSG_SELECT_EFFECTYN),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, xyz, sumtype, tp| c.is_set_card_full(duel, &[SET_RYZEAL], Some(xyz), sumtype, tp));
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |_, a| a.rp == 1 - a.tp);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, destg);
    e1.set_operation(duel, stock::destroy_first_target);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_LEAVE_GRAVE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, attachtg);
    e2.set_operation(duel, attachop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EFFECT_DESTROY_REPLACE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, reptg);
    e3.set_operation(duel, repop);
    e3.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(repfilter(c, duel, e.get_handler_player(duel)))),
    );
    c.register_effect(duel, e3);
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    Ok(false)
}

fn attachfilter(c: CardId, duel: &Duel, xyzc: CardId, tp: u8) -> bool {
    c.is_monster(duel) && c.is_can_be_xyz_material(duel, Some(xyzc), tp, REASON_EFFECT)
}

fn attachtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let c = a.e.get_handler(duel);
        return Ok(duel.is_existing_matching_card(|tc, duel| attachfilter(tc, duel, c, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, None, 1, tp, 0);
    Ok(false)
}

fn attachop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_XMATERIAL);
    let g = duel.select_matching_card(tp, |tc, duel| attachfilter(tc, duel, c, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        duel.hint_selection(tc);
        duel.overlay(c, tc)?;
    }
    Ok(())
}

fn repfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_location(duel, LOCATION_MZONE)
        && c.is_controler(duel, tp)
        && c.is_type(duel, TYPE_XYZ)
        && c.is_faceup(duel)
        && c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
        && !c.is_reason(duel, REASON_REPLACE)
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(a
            .eg
            .is_some_and(|eg| eg.is_exists(duel, |tc, duel| repfilter(tc, duel, tp), 1, None))
            && c.check_remove_overlay_card(duel, tp, 1, REASON_EFFECT));
    }
    duel.select_effect_yes_no_full(tp, c, 96)
}

fn repop(duel: &mut Duel, a: &Args) -> Script<()> {
    a.e.get_handler(duel).remove_overlay_card(duel, a.tp, 1, 1, REASON_EFFECT)?;
    Ok(())
}
