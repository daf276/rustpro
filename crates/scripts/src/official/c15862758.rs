// Number iC1000: Numerounius Numerounia
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 15862758;
pub const XYZ_NUMBER: u32 = 1000;

pub const LISTED_NAMES: &[u32] = &[89477759];

// Special Summoned by Number C1000: Numerounius, which the opponent's Raigeki destroys, with the opponent's monsters to attack.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (ID, MZONE),
    (89477759, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(ID)];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(13);
    const CT: usize = 5;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        c.is_summon_type(duel, &[SUMMON_TYPE_SPECIAL + 0x53b]) && !duel.is_turn_player(a.e.get_handler_player(duel))
    });
    e1.set_value(duel, 100000);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_MUST_ATTACK);
    e3.set_target_range(duel, 0, LOCATION_MZONE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, |duel, a| atcon(duel, a.e));
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EFFECT_MUST_ATTACK_MONSTER);
    e4.set_value(duel, Value::Card(|duel, e, c| i32::from(c == e.get_handler(duel))));
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e5.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_DELAY + EFFECT_FLAG_UNCOPYABLE);
    e5.set_range(duel, LOCATION_MZONE);
    e5.set_code(duel, EVENT_PHASE + PHASE_END);
    e5.set_count_limit(duel, 1, 0);
    e5.set_condition(duel, wincon);
    e5.set_operation(duel, |duel, a| {
        duel.win(a.tp, WIN_REASON_NUMBER_IC1000);
        Ok(())
    });
    c.register_effect(duel, e5);
    let e7 = duel.create_effect(c);
    e7.set_description(duel, aux::stringid(ID, 0));
    e7.set_category(duel, CATEGORY_RECOVER);
    e7.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e7.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e7.set_range(duel, LOCATION_MZONE);
    e7.set_condition(duel, nacon);
    e7.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e7.set_target(duel, natg);
    e7.set_operation(duel, naop);
    c.register_effect(duel, e7);
}

fn atcon(duel: &Duel, e: EffectId) -> bool {
    let c = e.get_handler(duel);
    c.is_summon_type(duel, &[SUMMON_TYPE_SPECIAL + 0x53b]) && duel.get_turn_count() > c.get_turn_id(duel)
}

fn wincon(duel: &Duel, a: &Args) -> bool {
    !duel.is_turn_player(a.tp) && a.e.get_handler(duel).get_battled_group_count(duel) == 0 && atcon(duel, a.e)
}

fn nacon(duel: &Duel, a: &Args) -> bool {
    duel.get_attacker().is_some_and(|at| !at.is_controler(duel, a.tp))
}

fn natg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let Some(at) = duel.get_attacker() else { return Ok(false) };
    if chk == 0 {
        return Ok(at.is_on_field(duel));
    }
    let atk = at.get_attack(duel);
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, a.tp, atk);
    Ok(false)
}

fn naop(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.negate_attack()?
        && let Some(at) = duel.get_attacker()
    {
        let atk = at.get_attack(duel).max(0) as u32;
        duel.recover(a.tp, atk, REASON_EFFECT)?;
    }
    Ok(())
}
