// Number 47: Nightmare Shark
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 31320433;
pub const XYZ_NUMBER: u32 = 47;

// In the Extra Deck, Xyz Summoned with Level 3 WATER Normal Monsters (Water Element, Aqua Snake) to attach another from the hand, and on the field
// with Xyz Import to attach Sangan to detach.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (14602126, HAND),
    (14602126, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (3732747, MZONE),
    (3732747, MZONE),
    (12436646, MZONE),
    (12436646, MZONE),
    (12436646, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(3);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_target(duel, mattg);
    e1.set_operation(duel, matop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, |duel, _| duel.is_able_to_enter_bp());
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, datg);
    e2.set_operation(duel, daop);
    c.register_effect(duel, e2);
}

fn matfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_location(duel, LOCATION_HAND) || c.is_faceup(duel))
        && !c.is_type(duel, TYPE_TOKEN)
        && c.is_level(duel, &[3])
        && c.is_attribute(duel, ATTRIBUTE_WATER)
}

fn mattg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && duel.is_existing_matching_card(matfilter, a.tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, None))
}

fn matop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_XMATERIAL);
    let g = duel.select_matching_card(tp, matfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.overlay(c, g)?;
    }
    Ok(())
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_attribute(duel, ATTRIBUTE_WATER) && c.is_has_effect(duel, EFFECT_CANNOT_DIRECT_ATTACK).is_none()
}

fn datg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    let g = duel.select_target(tp, filter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    let field_id = g.and_then(|g| g.get_first()).map_or(0, |tc| tc.get_field_id(duel));
    e.set_label(duel, i64::from(field_id));
    Ok(false)
}

fn daop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
    {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3205);
        e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_DIRECT_ATTACK);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
    }
    let label = e.get_label(duel);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_ATTACK);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_target_card(duel, |duel, e, c| e.get_label(duel) != i64::from(c.get_field_id(duel)));
    e2.set_label(duel, label);
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e2, tp);
    Ok(())
}
