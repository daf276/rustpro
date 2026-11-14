// Go! - D/D/D Divine Zero King Rage
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, PZONE};

const ID: u32 = 40227329;

pub const LISTED_SERIES: &[u16] = &[SET_DD];
// In the Pendulum Zone to Normal Summon the copies in hand without Tributes; on the field with D/D Pandora to Tribute.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (32146097, MZONE),
    (32146097, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1)), Check::TributeSummoned(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CHANGE_DAMAGE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_target_range(duel, 1, 0);
    e1.set_value(duel, Value::ChangeDamage(damval));
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_NO_EFFECT_DAMAGE);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_SUMMON_PROC);
    e3.set_range(duel, LOCATION_PZONE);
    e3.set_target_range(duel, LOCATION_HAND, 0);
    e3.set_count_limit(duel, 1, ID);
    e3.set_condition(duel, ntcon);
    e3.set_target_card(duel, |duel, _, c| c.is_level_above(duel, 5) && c.is_set_card(duel, &[SET_DD]));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_cost(duel, cost);
    e4.set_target(duel, target);
    e4.set_operation(duel, operation);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 5));
    e5.set_category(duel, CATEGORY_ATKCHANGE);
    e5.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e5.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e5.set_condition(duel, |duel, a| duel.get_lp(1 - a.tp) <= 4000);
    e5.set_operation(duel, atkop);
    c.register_effect(duel, e5);
    let e6 = duel.create_effect(c);
    e6.set_type(duel, EFFECT_TYPE_SINGLE);
    e6.set_code(duel, EFFECT_AVOID_BATTLE_DAMAGE);
    e6.set_value(duel, 1);
    c.register_effect(duel, e6);
    let e7 = duel.create_effect(c);
    e7.set_type(duel, EFFECT_TYPE_SINGLE);
    e7.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e7.set_value(duel, 1);
    c.register_effect(duel, e7);
}

fn damval(duel: &mut Duel, e: EffectId, _: Option<EffectId>, val: u32, r: u32, _: u8, _: Option<CardId>) -> u32 {
    let tp = e.get_handler_player(duel);
    if r & REASON_EFFECT != 0 && duel.get_flag_effect(tp, ID) == 0 {
        duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
        return 0;
    }
    val
}

fn ntcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    a.min == 0 && duel.get_location_count(c.get_controler(duel), LOCATION_MZONE) > 0
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, aux::TRUE, (1, 1), false, None, Some(c.into())));
    }
    let g = duel.select_release_group_cost(tp, aux::TRUE, (1, 1), false, None, Some(c.into()))?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn choices(duel: &Duel, e: EffectId, tp: u8) -> [bool; 3] {
    [
        duel.is_able_to_enter_bp() && e.get_handler(duel).is_has_effect(duel, EFFECT_DIRECT_ATTACK).is_none(),
        duel.get_flag_effect(1 - tp, ID + 1) == 0,
        duel.get_flag_effect(1 - tp, ID + 2) == 0,
    ]
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let [b1, b2, b3] = choices(duel, a.e, a.tp);
    Ok(chk == 0 && (b1 || b2 || b3))
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let [b1, b2, b3] = choices(duel, e, tp);
    let dtab: Vec<u64> = [b1, b2, b3]
        .into_iter()
        .zip(2..)
        .filter(|&(b, _)| b)
        .map(|(_, n)| aux::stringid(ID, n))
        .collect();
    // The Lua errors selecting among no options.
    unported!(duel, dtab.is_empty());
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RESOLVEEFFECT);
    let mut op = duel.select_option(tp, &dtab)? + 1;
    if !(b1 || b2) {
        op = 3;
    }
    if !(b1 || b3) {
        op = 2;
    }
    if b1 && b3 && !b2 && op == 2 {
        op = 3;
    }
    if b2 && b3 && !b1 {
        op += 1;
    }
    match op {
        1 => {
            if !c.is_relate_to_effect(duel, e) {
                return Ok(());
            }
            let e1 = duel.create_effect(c);
            e1.set_description(duel, 3205);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_DIRECT_ATTACK);
            e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
            e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
            c.register_effect(duel, e1);
        }
        2 | 3 => {
            let n = op as u32 - 1;
            duel.register_flag_effect(1 - tp, ID + n, RESET_PHASE | PHASE_END, 0, 1);
            aux::register_client_hint(duel, c, 0, tp, 0, 1, Some(aux::stringid(ID, u64::from(n) + 5)));
            let e2 = duel.create_effect(c);
            e2.set_type(duel, EFFECT_TYPE_FIELD);
            e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
            e2.set_code(duel, EFFECT_CANNOT_ACTIVATE);
            e2.set_target_range(duel, 0, 1);
            e2.set_value(duel, Value::EffectPlayer(if op == 2 { aclimit1 } else { aclimit2 }));
            e2.set_reset(duel, RESET_PHASE | PHASE_END);
            duel.register_effect(e2, tp);
        }
        _ => {}
    }
    Ok(())
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let lp = duel.get_lp(1 - a.tp);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
    e1.set_value(duel, lp);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    c.register_effect(duel, e1);
    Ok(())
}

fn aclimit1(duel: &Duel, _: EffectId, re: EffectId, _: u8) -> bool {
    re.get_handler(duel).is_location(duel, LOCATION_SZONE)
}

fn aclimit2(duel: &Duel, _: EffectId, re: EffectId, _: u8) -> bool {
    re.get_handler(duel).is_location(duel, LOCATION_HAND | LOCATION_GRAVE)
}
