// Spirit Reaper
use engine::constants::MSG_BECOME_TARGET;

use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 23205979;

// Raigeki to clear the opponent for a direct attack; an opponent's Mage Power to target it and make it self-destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (12580477, HAND),
    (12580477, HAND),
    (83746708, HAND | OPPONENT),
    (83746708, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Message(MSG_BECOME_TARGET)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_HANDES);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_BATTLE_DAMAGE);
    e2.set_condition(duel, |duel, a| a.ep != a.tp && duel.get_attack_target().is_none());
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_code(duel, EFFECT_SELF_DESTROY);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).get_owner_target_count(duel) > 0);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_code(duel, EVENT_CHAIN_SOLVED);
    e4.set_operation(duel, desop1);
    c.register_effect(duel, e4);
    let e5 = e4.clone_effect(duel);
    e5.set_code(duel, EVENT_BATTLED);
    e5.set_operation(duel, desop2);
    c.register_effect(duel, e5);
    let e6 = e4.clone_effect(duel);
    e6.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e6.set_code(duel, EVENT_BECOME_TARGET);
    e6.set_operation(duel, register);
    c.register_effect(duel, e6);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, 1 - a.tp, 1);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_field_group(a.ep, LOCATION_HAND, 0);
    if g.is_empty() {
        return Ok(());
    }
    let sg = g.random_select(duel, 1 - a.tp, 1);
    duel.send_to_grave(sg, REASON_DISCARD | REASON_EFFECT)?;
    Ok(())
}

fn register(duel: &mut Duel, a: &Args) -> Script<()> {
    a.e.get_handler(duel)
        .register_flag_effect_full(duel, ID + 1, RESET_CHAIN, 0, 1, i64::from(a.ev), 0);
    Ok(())
}

fn desop1(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(g) = duel.get_chain_info::<Option<CardSet>>(a.ev as u8, CHAININFO_TARGET_CARDS) else {
        return Ok(());
    };
    if !g.is_contains(c) {
        return Ok(());
    }
    if c.get_flag_effect_labels(duel, ID + 1).contains(&i64::from(a.ev)) {
        if duel.get_current_phase() & (PHASE_DAMAGE | PHASE_DAMAGE_CAL) != 0 && !duel.is_damage_calculated() {
            c.register_flag_effect(duel, ID, RESET_PHASE | PHASE_DAMAGE | RESETS_STANDARD, 0, 1);
        } else if c.is_has_effect(duel, EFFECT_DISABLE).is_none() && !c.is_disabled(duel) {
            duel.destroy(c, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn desop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.has_flag_effect(duel, ID) && c.is_has_effect(duel, EFFECT_DISABLE).is_none() && !c.is_disabled(duel) {
        duel.destroy(c, REASON_EFFECT)?;
    }
    Ok(())
}
