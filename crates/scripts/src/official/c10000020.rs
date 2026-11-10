// Slifer the Sky Dragon
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 10000020;

// In the hand to be Tribute Summoned with three Prevent Rats, and in the Graveyard for Monster Reborn to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (ID, GRAVE),
    (83764718, HAND),
    (83764718, HAND),
    (549481, MZONE),
    (549481, MZONE),
    (549481, MZONE),
    (549481, MZONE),
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
    e5.set_description(duel, aux::stringid(ID, 0));
    e5.set_category(duel, CATEGORY_TOGRAVE);
    e5.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e5.set_range(duel, LOCATION_MZONE);
    e5.set_count_limit(duel, 1, 0);
    e5.set_code(duel, EVENT_PHASE + PHASE_END);
    e5.set_condition(duel, |duel, a| a.e.get_handler(duel).is_special_summoned(duel));
    e5.set_target(duel, tgtg);
    e5.set_operation(duel, tgop);
    c.register_effect(duel, e5);
    let e6 = duel.create_effect(c);
    e6.set_type(duel, EFFECT_TYPE_SINGLE);
    e6.set_code(duel, EFFECT_UPDATE_ATTACK);
    e6.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e6.set_range(duel, LOCATION_MZONE);
    e6.set_value(
        duel,
        Value::Card(|duel, _, c| duel.get_field_group_count(c.get_controler(duel), LOCATION_HAND, 0) as i32 * 1000),
    );
    c.register_effect(duel, e6);
    let e7 = e6.clone_effect(duel);
    e7.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e7);
    let e8 = duel.create_effect(c);
    e8.set_description(duel, aux::stringid(ID, 1));
    e8.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DESTROY);
    e8.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e8.set_range(duel, LOCATION_MZONE);
    e8.set_code(duel, EVENT_SUMMON_SUCCESS);
    e8.set_condition(duel, atkcon);
    e8.set_target(duel, atktg);
    e8.set_operation(duel, atkop);
    c.register_effect(duel, e8);
    let e9 = e8.clone_effect(duel);
    e9.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e9);
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

fn atkfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_controler(duel, 1 - tp) && c.is_position(duel, POS_FACEUP_ATTACK)
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    let (eg, tp) = (a.eg.unwrap_or_default(), a.tp);
    !eg.is_contains(a.e.get_handler(duel)) && eg.is_exists(duel, |c, duel| atkfilter(c, duel, tp), 1, None)
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(a.e.get_handler(duel).is_relate_to_effect(duel, a.e));
    }
    let tp = a.tp;
    let g = a.eg.unwrap_or_default().filter(duel, |c, duel| atkfilter(c, duel, tp), None);
    duel.set_target_card(g);
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_target_cards(a.e).unwrap_or_default().filter(duel, CardId::is_faceup, None);
    if g.is_empty() {
        return Ok(());
    }
    let mut dg = CardSet::default();
    let c = a.e.get_handler(duel);
    for tc in g {
        let preatk = tc.get_attack(duel);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, -2000);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
        if preatk != 0 && tc.get_attack(duel) == 0 {
            dg.insert(tc);
        }
    }
    if dg.is_empty() {
        return Ok(());
    }
    duel.break_effect()?;
    duel.destroy(dg, REASON_EFFECT)?;
    Ok(())
}
