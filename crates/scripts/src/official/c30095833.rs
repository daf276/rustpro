// Odd-Eyes Rebellion Xyz Dragon
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT, PZONE};

const ID: u32 = 30095833;

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_type_full(duel, TYPE_PENDULUM, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(7);
    const CT: usize = 2;
}

pub const PENDULUM_LEVEL: u32 = 7;

// In the Pendulum Zone and on the field with Xyz Import to attach Sangan, DARK Sangan in the Graveyard to attach, and Sangan and
// Dark Hole on the opponent's side to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (ID, MZONE),
    (14602126, HAND),
    (14602126, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, GRAVE),
    (26202165, GRAVE),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Pendulum::add_procedure_full(duel, c, false, None);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_LEAVE_GRAVE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e2.set_code(duel, EFFECT_CANNOT_BE_XYZ_MATERIAL);
    e2.set_condition(duel, matcon);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e3.set_target(duel, destg);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_DELAY);
    e4.set_code(duel, EVENT_DESTROYED);
    e4.set_condition(duel, pencon);
    e4.set_target(duel, pentg);
    e4.set_operation(duel, penop);
    c.register_effect(duel, e4);
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, c, tp) = (a.e, a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    duel.set_possible_operation_info(0, CATEGORY_LEAVE_GRAVE, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn ovfilter(c: CardId, duel: &Duel, xc: CardId, tp: u8) -> bool {
    c.is_attribute(duel, ATTRIBUTE_DARK) && c.is_can_be_xyz_material(duel, Some(xc), tp, REASON_EFFECT)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c, tp) = (a.e, a.e.get_handler(duel), a.tp);
    if !c.is_relate_to_effect(duel, e) || duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? == 0 {
        return Ok(());
    }
    let g = duel.get_matching_group(
        aux::necro_valley_filter(|oc, duel| ovfilter(oc, duel, c, tp)),
        tp,
        LOCATION_GRAVE,
        0,
        None,
    );
    if g.is_empty() || !duel.select_yes_no(tp, aux::stringid(ID, 3))? {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATTACH);
    let og = g.select(duel, tp, 1, 1, None)?;
    if !og.is_empty() {
        duel.break_effect()?;
        duel.overlay(c, og)?;
    }
    Ok(())
}

fn matcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_status(duel, STATUS_SPSUMMON_TURN) && c.is_xyz_summoned(duel)
}

fn desfilter() -> impl Fn(CardId, &Duel) -> bool {
    aux::faceup_filter(|c, duel| c.is_attack_below(duel, 3000))
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE)
            && chkc.is_controler(duel, 1 - tp)
            && chkc.is_faceup(duel)
            && chkc.is_attack_below(duel, 3000));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(desfilter(), tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, desfilter(), tp, 0, LOCATION_MZONE, 1, 2, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, g.map_or(0, |g| g.len() as u8), 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}

fn pencon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_location(duel, LOCATION_MZONE) && c.is_faceup(duel)
}

fn pentg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && duel.check_pendulum_zones(a.tp))
}

fn penop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !duel.check_pendulum_zones(tp) {
        return Ok(());
    }
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.move_to_field(c, tp, tp, LOCATION_PZONE, POS_FACEUP, true)?;
    }
    Ok(())
}
