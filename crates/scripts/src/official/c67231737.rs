// Link Bumper
use super::c49847524::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 67231737;
// s[tp], the monsters that attacked this turn, at this key plus tp.
const ATTACKERS: u64 = (ID as u64) << 16;

// On the field and in the Extra Deck, with Cyberse monsters it points to and the opponent's monsters for them to attack.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, EXTRA),
    (24154052, MZONE),
    (32295838, MZONE),
    (36211150, MZONE),
    (47226949, MZONE),
    (24154052, MZONE | OPPONENT),
    (32295838, MZONE | OPPONENT),
    (36211150, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_DAMAGE_STEP_END);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, condition);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_ATTACK_ANNOUNCE);
        ge1.set_operation(duel, checkop);
        duel.register_effect(ge1, 0);
        aux::add_values_reset(duel, |duel| {
            duel.set_script_state(ATTACKERS, 0);
            duel.set_script_state(ATTACKERS | 1, 0);
        });
    });
}

fn checkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = a.eg.and_then(|eg| eg.get_first()) else {
        return Ok(());
    };
    if tc.get_flag_effect(duel, ID) == 0 {
        let key = ATTACKERS | u64::from(a.ep);
        duel.set_script_state(key, duel.get_script_state(key) + 1);
        tc.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
    }
    Ok(())
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let Some(attacker) = duel.get_attacker() else { return false };
    if attacker.is_controler(duel, 1 - a.tp) {
        return false;
    }
    let lg = a.e.get_handler(duel).get_linked_group(duel);
    lg.is_contains(attacker) && attacker.get_battle_target(duel).is_some_and(|d| d.is_controler(duel, 1 - a.tp))
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let Some(attacker) = duel.get_attacker() else { return Ok(false) };
    if chk == 0 {
        return Ok(duel.get_script_state(ATTACKERS | u64::from(tp)) == 0 || attacker.get_flag_effect(duel, ID) != 0);
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK);
    e1.set_property(duel, EFFECT_FLAG_OATH + EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, e, c| e.get_label(duel) != i64::from(c.get_field_id(duel)));
    e1.set_label(duel, attacker.get_field_id(duel).into());
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(false)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_link_monster(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.get_matching_group_count(filter, a.tp, LOCATION_MZONE, 0, Some(a.e.get_handler(duel).into())) > 0);
    }
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let gc = duel.get_matching_group_count(filter, tp, LOCATION_MZONE, 0, Some(c.into()));
    if gc == 0 {
        return Ok(());
    }
    let Some(attacker) = duel.get_attacker() else { return Ok(()) };
    if attacker.is_relate_to_battle(duel) && attacker.is_faceup(duel) {
        let e0 = duel.create_effect(c);
        e0.set_type(duel, EFFECT_TYPE_SINGLE);
        e0.set_code(duel, EFFECT_EXTRA_ATTACK_MONSTER);
        e0.set_value(duel, gc as i32);
        e0.set_reset(duel, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_BATTLE);
        attacker.register_effect(duel, e0);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_code(duel, EFFECT_MUST_ATTACK_MONSTER);
        e1.set_target_range(duel, LOCATION_MZONE, 0);
        e1.set_value(duel, Value::Card(|duel, _, c| i32::from(filter(c, duel))));
        e1.set_reset(duel, RESET_PHASE | PHASE_BATTLE);
        duel.register_effect(e1, tp);
    }
    Ok(())
}
