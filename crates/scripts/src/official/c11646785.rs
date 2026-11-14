// Super Quantal Mech Beast Aeroboros
use super::c57031794::{attachop, attachtg};
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 11646785;

pub const LISTED_SERIES: &[u16] = &[SET_SUPER_QUANTUM];
pub const LISTED_NAMES: &[u32] = &[85374678];

// In the Extra Deck, Xyz Summoned with two Level 4 monsters (Hyosube, Water Omotics), and Super Quantum Green Layer and Red
// Layer in the hand to attach for the Quick effect.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (2118022, MZONE),
    (2118022, MZONE),
    (2483611, MZONE),
    (2483611, MZONE),
    (85374678, HAND),
    (85374678, HAND),
    (59975920, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).get_overlay_count(duel) == 0);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_POSITION + CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_SINGLE);
    e2.set_condition(duel, |duel, a| !quickeffcond(duel, a));
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, settg);
    e2.set_operation(duel, setop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e3.set_condition(duel, quickeffcond);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, 0);
    e4.set_target(duel, attachtg);
    e4.set_operation(duel, attachop);
    c.register_effect(duel, e4);
}

fn quickeffcond(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel)
        .get_overlay_group(duel)
        .is_exists(duel, |c, duel| c.is_code(duel, 85374678), 1, None)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_can_turn_set(duel) && chkc != c);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_can_turn_set, tp, LOCATION_MZONE, LOCATION_MZONE, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
    let g = duel.select_target(
        tp,
        CardId::is_can_turn_set,
        tp,
        LOCATION_MZONE,
        LOCATION_MZONE,
        1,
        1,
        Some(c.into()),
    )?;
    duel.set_operation_info(0, CATEGORY_POSITION, g, 1, tp, POS_FACEDOWN_DEFENSE as i32);
    Ok(false)
}

pub fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_faceup(duel)
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.change_position(tc, POS_FACEDOWN_DEFENSE)?;
    }
    Ok(())
}
