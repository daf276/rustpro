// Number 88: Gimmick Puppet of Leo
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 48995978;

pub const COUNTER_PLACE_LIST: &[u16] = &[0x2b];
pub const XYZ_NUMBER: u32 = 88;
const COUNTER_DESTINY: u16 = 0x2b;

// On the field with Xyz Import to attach Sangan, which Leo detaches to place a Destiny Counter; the third counter, and so the win, is not reached.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (14602126, HAND),
    (14602126, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(8);
    const CT: usize = 3;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    c.enable_counter_permit(duel, COUNTER_DESTINY);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_COUNTER);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, |duel, a| duel.get_field_group_count(a.tp, LOCATION_STZONE, 0) == 0);
    e1.set_cost(duel, ctcost);
    e1.set_target(duel, cttg);
    e1.set_operation(duel, ctop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_property(duel, EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_CHAIN_SOLVING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_operation(duel, winop);
    c.register_effect(duel, e2);
}

fn ctcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_phase(PHASE_MAIN1));
    }
    let tp = a.tp;
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_BP);
    e1.set_target_range(duel, 1, 0);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(false)
}

fn cttg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.check_remove_overlay_card(duel, tp, 1, REASON_EFFECT) && c.is_can_add_counter(duel, COUNTER_DESTINY, 1));
    }
    duel.set_operation_info(0, CATEGORY_COUNTER, None, 1, tp, i32::from(COUNTER_DESTINY));
    Ok(false)
}

fn ctop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && c.is_faceup(duel) && c.remove_overlay_card(duel, tp, 1, 1, REASON_EFFECT)? > 0 {
        c.add_counter(duel, COUNTER_DESTINY, 1);
    }
    Ok(())
}

fn winop(duel: &mut Duel, a: &Args) -> Script<()> {
    if a.e.get_handler(duel).get_counter(duel, COUNTER_DESTINY) == 3 {
        duel.win(a.tp, WIN_REASON_PUPPET_LEO);
    }
    Ok(())
}
