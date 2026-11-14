// Super Quantal Mech Beast Grampulse
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 85252081;

pub const LISTED_SERIES: &[u16] = &[SET_SUPER_QUANTUM];

const BLUE_LAYER: u32 = 12369277;

pub const LISTED_NAMES: &[u32] = &[BLUE_LAYER];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(3);
    const CT: usize = 2;
}

// Blue Layer in the hand to attach, in the Extra Deck, Xyz Summoned with Level 3 Normal Monsters, and the opponent's Trap to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (6297941, MZONE),
    (6297941, MZONE),
    (BLUE_LAYER, HAND),
    (BLUE_LAYER, HAND),
    (4206964, SZONE | OPPONENT),
    (4206964, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).get_overlay_count(duel) == 0);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_SINGLE);
    e2.set_condition(duel, |duel, a| !quickeffcond(duel, a));
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, destg);
    e2.set_operation(duel, stock::destroy_first_target);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_hint_timing(duel, 0, TIMING_END_PHASE | TIMING_EQUIP);
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
        .is_exists(duel, |c, duel| c.is_code(duel, BLUE_LAYER), 1, None)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_ONFIELD) && chkc.is_spell_trap(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_spell_trap, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, CardId::is_spell_trap, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    Ok(false)
}

fn attachfilter(c: CardId, duel: &Duel, tp: u8, xc: CardId) -> bool {
    c.is_set_card(duel, &[SET_SUPER_QUANTUM])
        && c.is_monster(duel)
        && !c.is_type(duel, TYPE_TOKEN)
        && (c.is_location(duel, LOCATION_HAND) || c.is_faceup(duel))
        && c.is_can_be_xyz_material(duel, Some(xc), tp, REASON_EFFECT)
}

pub fn attachtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    Ok(chk == 0
        && c.is_type(duel, TYPE_XYZ)
        && duel.is_existing_matching_card(
            |tc, duel| attachfilter(tc, duel, tp, c),
            tp,
            LOCATION_HAND | LOCATION_MZONE,
            0,
            1,
            None,
        ))
}

pub fn attachop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || c.is_facedown(duel) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_XMATERIAL);
    let g = duel.select_matching_card(
        tp,
        |tc, duel| attachfilter(tc, duel, tp, c),
        tp,
        LOCATION_HAND | LOCATION_MZONE,
        0,
        1,
        1,
        None,
    )?;
    if let Some(tc) = g.get_first()
        && !tc.is_immune_to_effect(duel, e)
    {
        duel.overlay_full(c, tc, true)?;
    }
    Ok(())
}
