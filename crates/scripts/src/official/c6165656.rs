// Number C88: Gimmick Puppet Disaster Leo
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 6165656;

pub const LISTED_SERIES: &[u16] = &[SET_RANK_UP_MAGIC];
pub const XYZ_NUMBER: u32 = 88;

pub const LISTED_NAMES: &[u32] = &[48995978];

// On the field with Xyz Import to attach Sangan, which Disaster Leo detaches to burn the opponent; the win at 2000 LP or less is not reached.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (14602126, HAND),
    (14602126, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_value(duel, Value::SpSummonCondition(splimit));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_DAMAGE);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e3.set_target(duel, target);
    e3.set_operation(duel, stock::damage);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e4.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_code(duel, EVENT_PHASE + PHASE_END);
    e4.set_count_limit(duel, 1, 0);
    e4.set_condition(duel, wincon);
    e4.set_operation(duel, winop);
    c.register_effect(duel, e4);
}

fn splimit(duel: &Duel, _: EffectId, se: Option<EffectId>, _: u8, _: u32, _: u8, _: u8) -> bool {
    unported!(duel, se.is_none());
    se.is_some_and(|se| {
        let sc = se.get_handler(duel);
        sc.is_set_card(duel, &[SET_RANK_UP_MAGIC]) && sc.is_spell(duel)
    })
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(1 - tp);
    duel.set_target_param(1000);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 1000);
    Ok(false)
}

fn wincon(duel: &Duel, a: &Args) -> bool {
    let tp = a.e.get_handler_player(duel);
    duel.is_turn_player(tp) && duel.get_lp(1 - tp) <= 2000 && a.e.get_handler(duel).get_overlay_count(duel) == 0
}

fn winop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.win(a.tp, WIN_REASON_DISASTER_LEO);
    Ok(())
}
