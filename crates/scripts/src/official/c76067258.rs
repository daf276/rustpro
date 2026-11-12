// Number 66: Master Key Beetle
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};
use engine::constants::MSG_SELECT_EFFECTYN;

const ID: u32 = 76067258;
pub const XYZ_NUMBER: u32 = 66;

// In the Extra Deck, Xyz Summoned with Level 4 DARK Normal Monsters, copies on the field to target, and the opponent's Raigeki to
// destroy them.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (ID, MZONE),
    (7459013, MZONE),
    (11321183, MZONE),
    (7359741, MZONE),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Message(MSG_SELECT_EFFECTYN)];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_attribute_full(duel, ATTRIBUTE_DARK, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_code(duel, EFFECT_DESTROY_REPLACE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target(duel, reptg);
    c.register_effect(duel, e3);
}

fn filter(c: CardId, duel: &Duel, ec: CardId) -> bool {
    !ec.is_has_card_target(duel, c)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, tp) && chkc != c && filter(chkc, duel, c));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|tc, duel| filter(tc, duel, c), tp, LOCATION_ONFIELD, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, |tc, duel| filter(tc, duel, c), tp, LOCATION_ONFIELD, 0, 1, 1, Some(c.into()))?;
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, e) && tc.is_relate_to_effect(duel, e) {
        c.set_card_target(duel, tc);
        tc.register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD, 0, 0);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
        e2.set_property(duel, EFFECT_FLAG_SET_AVAILABLE + EFFECT_FLAG_OWNER_RELATE);
        e2.set_condition(duel, indcon);
        e2.set_value(duel, 1);
        e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e2);
    }
    Ok(())
}

fn indcon(duel: &Duel, a: &Args) -> bool {
    let owner = a.e.get_owner(duel);
    owner.is_has_card_target(duel, a.e.get_handler(duel)) && owner.is_on_field(duel) && owner.is_faceup(duel)
}

fn repfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_controler(duel, tp) && c.get_flag_effect(duel, ID) != 0
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(
            !c.is_reason(duel, REASON_REPLACE) && c.get_card_target(duel).is_exists(duel, |tc, duel| repfilter(tc, duel, tp), 1, None)
        );
    }
    if duel.select_effect_yes_no_full(tp, c, 96)? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
        let g = c
            .get_card_target(duel)
            .filter_select(duel, tp, |tc, duel| repfilter(tc, duel, tp), 1, 1, None)?;
        duel.send_to_grave(g, REASON_EFFECT)?;
        return Ok(true);
    }
    Ok(false)
}
