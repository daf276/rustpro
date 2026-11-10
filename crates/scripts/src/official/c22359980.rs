// Mirror Wall
use crate::prelude::*;
use crate::testing::{Check, HAND, SZONE};

const ID: u32 = 22359980;

// One in the hand to activate and one already set, both against the opponent's attackers.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, SZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::Message(engine::constants::MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_DAMAGE_STEP);
    e1.set_condition(duel, aux::stat_change_damage_step_condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_SET_ATTACK_FINAL);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_target_range(duel, 0, LOCATION_MZONE);
    e3.set_target_card(duel, atktg);
    e3.set_value(duel, Value::Card(|duel, _, c| (c.get_attack(duel) + 1) / 2));
    c.register_effect(duel, e3);
    // The Lua's group is shared by the three effects: it lives on e1, which each of them has as label object.
    for e in [e1, e2, e3] {
        e.set_label_object(duel, Some(e1));
    }
    e1.set_label_object_group(duel, Some(CardSet::default()));
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e4.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e4.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_count_limit(duel, 1, 0);
    e4.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e4.set_operation(duel, mtop);
    c.register_effect(duel, e4);
}

fn group(duel: &Duel, e: EffectId) -> CardSet {
    e.get_label_object(duel)
        .and_then(|holder| holder.get_label_object_group(duel))
        .unwrap_or_default()
}

fn set_group(duel: &mut Duel, e: EffectId, g: CardSet) {
    if let Some(holder) = e.get_label_object(duel) {
        holder.set_label_object_group(duel, Some(g));
    }
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    set_group(duel, a.e, CardSet::default());
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(attacker) = duel.get_attacker() else { return Ok(()) };
    if attacker.is_controler(duel, 1 - a.tp) && attacker.is_faceup(duel) && attacker.is_location(duel, LOCATION_MZONE) {
        let mut g = group(duel, a.e);
        g.insert(attacker);
        set_group(duel, a.e, g);
        if attacker.get_flag_effect(duel, ID) == 0 {
            attacker.register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD, 0, 1);
        }
    }
    Ok(())
}

fn atktg(duel: &Duel, e: EffectId, c: CardId) -> bool {
    c.get_flag_effect(duel, ID) != 0 && group(duel, e).is_contains(c)
}

fn mtop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.check_lp_cost(tp, 2000) && duel.select_yes_no(tp, aux::stringid(ID, 0))? {
        duel.pay_lp_cost(tp, 2000)?;
    } else {
        duel.destroy(a.e.get_handler(duel), REASON_COST)?;
    }
    Ok(())
}
