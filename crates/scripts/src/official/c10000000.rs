// Obelisk the Tormentor
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 10000000;

// In the hand to be Tribute Summoned with three Prevent Rats, in the Graveyard for Monster Reborn to Special Summon, and on the
// field to Tribute two Prevent Rats and destroy the opponent's two Sangan.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (ID, GRAVE),
    (ID, MZONE),
    (83764718, HAND),
    (83764718, HAND),
    (549481, MZONE),
    (549481, MZONE),
    (549481, MZONE),
    (549481, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::TributeSummoned(3),
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct SummonProcedure;

impl NormalProcedure for SummonProcedure {
    const NS: bool = true;
    const OPT: bool = false;
    const MIN: i32 = 3;
    const MAX: i32 = 3;
}

struct SetProcedure;

impl NormalProcedure for SetProcedure {
    const NS: bool = false;
    const OPT: bool = false;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Normal::add_summon_procedure::<SummonProcedure>(duel, c);
    Normal::add_set_procedure::<SetProcedure>(duel, c);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_CANNOT_DISABLE_SUMMON);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e4.set_code(duel, EVENT_SUMMON_SUCCESS);
    e4.set_operation(duel, |duel, _| {
        duel.set_chain_limit_till_chain_end(|_, _, _, _, _| false, 0);
        Ok(())
    });
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_SINGLE);
    e5.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e5.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e5.set_range(duel, LOCATION_MZONE);
    e5.set_value(duel, 1);
    c.register_effect(duel, e5);
    let e6 = duel.create_effect(c);
    e6.set_description(duel, aux::stringid(ID, 0));
    e6.set_category(duel, CATEGORY_TOGRAVE);
    e6.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e6.set_range(duel, LOCATION_MZONE);
    e6.set_count_limit(duel, 1, 0);
    e6.set_code(duel, EVENT_PHASE + PHASE_END);
    e6.set_condition(duel, |duel, a| a.e.get_handler(duel).is_special_summoned(duel));
    e6.set_target(duel, tgtg);
    e6.set_operation(duel, tgop);
    c.register_effect(duel, e6);
    let e7 = duel.create_effect(c);
    e7.set_description(duel, aux::stringid(ID, 1));
    e7.set_category(duel, CATEGORY_DESTROY);
    e7.set_type(duel, EFFECT_TYPE_IGNITION);
    e7.set_range(duel, LOCATION_MZONE);
    e7.set_cost(duel, descost);
    e7.set_target(duel, destg);
    e7.set_operation(duel, desop);
    c.register_effect(duel, e7);
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let c = a.e.get_handler(duel);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && c.is_faceup(duel) {
        duel.send_to_grave(c, REASON_EFFECT)?;
    }
    Ok(())
}

fn descost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let dg = duel.get_matching_group(aux::TRUE, tp, 0, LOCATION_MZONE, None);
    // aux.ReleaseCheckTarget
    let check = |duel: &Duel, sg: CardSet, _: u8, _: CardSet| (dg.is_exists(duel, aux::TRUE, 1, Some(sg)), false);
    if chk == 0 {
        return Ok(
            c.get_attack_announced_count(duel) == 0 && duel.check_release_group_cost(tp, aux::TRUE, (2, 2), false, Some(&check), None)
        );
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_OATH);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK_ANNOUNCE);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    c.register_effect(duel, e1);
    let g = duel.select_release_group_cost(tp, aux::TRUE, (2, 2), false, Some(&check), None)?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(aux::TRUE, tp, 0, LOCATION_MZONE, 1, None));
    }
    let g = duel.get_matching_group(aux::TRUE, tp, 0, LOCATION_MZONE, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(aux::TRUE, a.tp, 0, LOCATION_MZONE, None);
    duel.destroy(g, REASON_EFFECT)?;
    Ok(())
}
