// Dark Contract with Errors
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, SZONE};

const ID: u32 = 37209439;

pub const LISTED_SERIES: &[u16] = &[SET_DD];
// Set and in the hand, with D/D monsters on the field for its negation.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (ID, HAND),
    (72291412, MZONE),
    (45206713, MZONE),
    (93317313, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, negcon);
    e2.set_operation(duel, negop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DAMAGE);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e3.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e3.set_target(duel, super::c46372010::damtg::<1000>);
    e3.set_operation(duel, super::c46372010::damop);
    c.register_effect(duel, e3);
    aux::double_snare_validity(duel, c, LOCATION_SZONE);
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(
        |c, duel| c.is_faceup(duel) && c.is_set_card(duel, &[SET_DD]),
        a.tp,
        LOCATION_MZONE,
        0,
        1,
        None,
    )
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let fid = c.get_field_id(duel).into();
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_DISABLE);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_target_range(duel, LOCATION_SZONE, LOCATION_SZONE);
    e1.set_target_card(duel, distg);
    e1.set_label(duel, fid);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_CHAIN_SOLVING);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_operation(duel, disop);
    e2.set_label(duel, fid);
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e2, tp);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_DISABLE_TRAPMONSTER);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e3.set_target_card(duel, distg);
    e3.set_label(duel, fid);
    e3.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e3, tp);
    Ok(())
}

fn distg(duel: &Duel, e: EffectId, c: CardId) -> bool {
    i64::from(c.get_field_id(duel)) != e.get_label(duel) && c.is_trap(duel)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let ev = a.ev as u8;
    let tl: u32 = duel.get_chain_info(ev, CHAININFO_TRIGGERING_LOCATION);
    if let Some(re) = a.re
        && tl == LOCATION_SZONE
        && re.is_trap_effect(duel)
        && i64::from(re.get_handler(duel).get_field_id(duel)) != a.e.get_label(duel)
    {
        duel.negate_effect(ev);
    }
    Ok(())
}
