// Mind Mirror Force
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 66247039;

// Set and in the hand, with Elfnote Lucina on the opponent's field to attack and be negated.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, HAND),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.register_flag_effect(duel, ID, 0, 0, 1);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD);
        ge1.set_code(duel, EFFECT_CANNOT_INACTIVATE);
        ge1.set_value(duel, Value::Chain(inactfilter));
        duel.register_effect(ge1, 0);
        let ge2 = ge1.clone_effect(duel);
        ge2.set_code(duel, EFFECT_CANNOT_DISEFFECT);
        duel.register_effect(ge2, 0);
    });
    let e0 = duel.create_effect(c);
    e0.set_description(duel, aux::stringid(ID, 0));
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e0.set_code(duel, EFFECT_TRAP_ACT_IN_SET_TURN);
    e0.set_condition(duel, |duel, a| {
        duel.get_matching_group_count(CardId::is_faceup, a.e.get_handler_player(duel), 0, LOCATION_ONFIELD, None) >= 5
    });
    c.register_effect(duel, e0);
    let e1a = duel.create_effect(c);
    e1a.set_description(duel, aux::stringid(ID, 1));
    e1a.set_category(duel, CATEGORY_DISABLE + CATEGORY_DESTROY);
    e1a.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1a.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e1a.set_condition(duel, |duel, a| condition1(duel, a.tp));
    e1a.set_target(duel, target);
    e1a.set_operation(duel, activate);
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_code(duel, EVENT_CHAINING);
    e1b.set_condition(duel, condition2);
    c.register_effect(duel, e1b);
    let e1c = e1b.clone_effect(duel);
    e1c.set_condition(duel, condition3);
    c.register_effect(duel, e1c);
}

fn inactfilter(duel: &Duel, _: EffectId, ch: u8) -> bool {
    let Some(trig_eff) = duel.get_chain_info::<Option<EffectId>>(ch, CHAININFO_TRIGGERING_EFFECT) else {
        return false;
    };
    let trig_player: u8 = duel.get_chain_info(ch, CHAININFO_TRIGGERING_PLAYER);
    trig_eff.get_handler(duel).has_flag_effect(duel, ID)
        && duel.get_matching_group_count(CardId::is_faceup, trig_player, 0, LOCATION_ONFIELD, None) >= 5
}

fn condition1(duel: &Duel, tp: u8) -> bool {
    let bc = duel.get_attacker();
    unported!(duel, bc.is_none());
    let Some(bc) = bc else { return false };
    bc.is_controler(duel, 1 - tp)
        && duel
            .get_matching_group(CardId::is_faceup, tp, 0, LOCATION_MZONE, None)
            .get_max_group(duel, |c, duel| i64::from(c.get_attack(duel)))
            .is_some_and(|(g, _)| g.is_contains(bc))
}

fn condition2(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    if duel.check_event_full(EVENT_ATTACK_ANNOUNCE).is_some() && condition1(duel, tp) {
        return false;
    }
    if !a.re.is_some_and(|re| a.rp == 1 - tp && re.is_monster_effect(duel)) {
        return false;
    }
    duel.get_operation_info(a.ev as u8, CATEGORY_DESTROY)
        .and_then(|info| info.op_cards)
        .is_some_and(|g| g.is_exists(duel, CardId::is_on_field, 1, None))
}

fn condition3(duel: &Duel, a: &Args) -> bool {
    if condition2(duel, a) {
        return false;
    }
    duel.is_turn_player(a.tp)
        && a.rp == 1 - a.tp
        && a.re.is_some_and(|re| re.is_monster_effect(duel))
        && chain::is_triggering_location(duel, a.ev as u8, LOCATION_HAND | LOCATION_MZONE)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(CardId::is_negatable, tp, 0, LOCATION_ONFIELD, None);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_DISABLE, Some(g), g.len() as u8, tp, 0);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, tp, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let mut g = duel.get_matching_group(CardId::is_negatable, tp, 0, LOCATION_ONFIELD, None);
    g.keep_matching(duel, |c, duel| c.is_can_be_disabled_by_effect(duel, e, true), None);
    if !g.is_empty() {
        for nc in g {
            nc.negate_effects(duel, c, 0, false);
        }
        duel.adjust_instantly(None);
        duel.destroy(g, REASON_EFFECT)?;
    }
    if !e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        return Ok(());
    }
    aux::register_client_hint_full(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 2)), 0, 2);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_DIRECT_ATTACK);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_reset_full(duel, RESET_PHASE | PHASE_END, 2);
    duel.register_effect(e1, tp);
    Ok(())
}
