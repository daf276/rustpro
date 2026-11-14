// Brotherhood of the Fire Fist - Peacock
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 20265095;

pub const LISTED_SERIES: &[u16] = &[SET_FIRE_FORMATION, SET_FIRE_FIST];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_FIRE_FIST], Some(lc), sumtype, tp)
    }
}

// On the field with Fire Formation - Tenki in the hand to activate and send, free Link zones, and weak monsters of the opponent to attack and take.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (57103969, HAND),
    (57103969, HAND),
    (57103969, HAND),
    (57103969, HAND),
    (57103969, HAND),
    (57103969, HAND),
    (24154052, MZONE | OPPONENT),
    (24154052, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CANNOT_BE_BATTLE_TARGET);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, tgcon);
    e1.set_value(duel, Value::Card(aux::imval1));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_CONTROL);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, |duel, a, chk| Cost::replaceable_full(duel, a, chk, ctcost, extracon));
    e2.set_target(duel, cttg);
    e2.set_operation(duel, ctop);
    c.register_effect(duel, e2);
}

fn lkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_FIRE_FIST]) && c.is_monster(duel)
}

fn tgcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).get_linked_group(duel).is_exists(duel, lkfilter, 1, None)
}

fn ctcostfilter(c: CardId, duel: &Duel, tp: u8, zones: u32) -> bool {
    c.is_faceup(duel)
        && c.is_set_card(duel, &[SET_FIRE_FORMATION])
        && c.is_spell_trap(duel)
        && c.is_able_to_grave_as_cost(duel)
        && duel.get_mzone_count_full(tp, Some(c.into()), tp, LOCATION_REASON_CONTROL, zones) > 0
}

fn ctcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let zones = a.e.get_handler(duel).get_linked_zone(duel) & ZONES_MMZ;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| ctcostfilter(c, duel, tp, zones), tp, LOCATION_ONFIELD, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, |c, duel| ctcostfilter(c, duel, tp, zones), tp, LOCATION_ONFIELD, 0, 1, 1, None)?;
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn extracon(duel: &Duel, _: EffectId, a: &Args) -> bool {
    let zones = a.e.get_handler(duel).get_linked_zone(duel) & ZONES_MMZ;
    duel.get_location_count_full(a.tp, LOCATION_MZONE, a.tp, LOCATION_REASON_CONTROL, zones) > 0
}

fn cttg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && chkc.is_controler_can_be_changed(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| c.is_controler_can_be_changed(duel), tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONTROL);
    let g = duel.select_target(tp, |c, duel| c.is_controler_can_be_changed(duel), tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_CONTROL, g, 1, 0, 0);
    Ok(false)
}

fn ctop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let zone = c.get_linked_zone(duel) & ZONES_MMZ;
    let reason_player = duel.get_reason_player();
    if duel.get_control_full(tc, tp, PHASE_END as u16, 1, zone, reason_player)? != 0 {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3206);
        e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CANNOT_ATTACK);
        e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD_PHASE_END & !RESET_TURN_SET));
        tc.register_effect(duel, e1);
    }
    Ok(())
}
