// Battlewasp - Hama the Conquering Bow
use super::c90587641::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 80949182;

pub const LISTED_SERIES: &[u16] = &[SET_BATTLEWASP];
// Copies on the field to attack the opponent's weak monsters, with Battlewasp monsters in the Graveyard and a copy in the Extra
// Deck, Synchro Summoned from Water Spirit (a Level 1 Tuner) and Dark Magician (Level 7).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, EXTRA),
    (487395, MZONE),
    (36996508, MZONE),
    (58192742, MZONE | OPPONENT),
    (58192742, MZONE | OPPONENT),
    (65899613, GRAVE),
    (65899613, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, tncon);
    e1.set_operation(duel, tnop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_MATERIAL_CHECK);
    e2.set_value(duel, Value::MaterialCheck(valcheck));
    e2.set_label_object(duel, Some(e1));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_ATKCHANGE);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_BATTLE_DAMAGE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_condition(duel, atcon);
    e3.set_target(duel, attg);
    e3.set_operation(duel, atop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_DAMAGE);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e4.set_code(duel, EVENT_PHASE | PHASE_BATTLE);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, (ID, 1));
    e4.set_condition(duel, damcon);
    e4.set_target(duel, damtg);
    e4.set_operation(duel, damop);
    c.register_effect(duel, e4);
    aux::global_check(duel, ID, |duel| {
        duel.set_script_state(STATE, 1);
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_BATTLE_DAMAGE);
        ge1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
        ge1.set_operation(duel, checkop);
        duel.register_effect(ge1, 0);
        let ge2 = duel.create_effect(c);
        ge2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge2.set_code(duel, EVENT_ADJUST);
        ge2.set_count_limit(duel, 1, 0);
        ge2.set_operation(duel, clear);
        duel.register_effect(ge2, 0);
    });
}

const STATE: u64 = (ID as u64) << 16;

fn valcheck(duel: &mut Duel, e: EffectId, c: CardId) {
    let g = c.get_material(duel);
    let label = i64::from(g.is_exists(duel, |c, duel| c.is_type(duel, TYPE_SYNCHRO), 1, None));
    if let Some(e1) = e.get_label_object(duel) {
        e1.set_label(duel, label);
    }
}

fn tncon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_synchro_summoned(duel) && a.e.get_label(duel) == 1
}

fn tnop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, 3201);
    e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_EXTRA_ATTACK);
    e1.set_value(duel, 1);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    c.register_effect(duel, e1);
    Ok(())
}

fn atcon(_: &Duel, a: &Args) -> bool {
    a.ep == 1 - a.tp
}

fn attg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let not_destroyed = |c: CardId, duel: &Duel| !(c.is_faceup(duel) && c.is_status(duel, STATUS_BATTLE_DESTROYED));
    Ok(chk == 0 && duel.is_existing_matching_card(not_destroyed, a.tp, 0, LOCATION_MZONE, 1, None))
}

fn atop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(CardId::is_faceup, a.tp, 0, LOCATION_MZONE, None);
    for sc in g {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        e1.set_value(duel, -1000);
        sc.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
        sc.register_effect(duel, e2);
    }
    Ok(())
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(a.tp) && duel.get_script_state(STATE) != 0
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_BATTLEWASP]) && c.is_monster(duel)
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    let dam = duel.get_matching_group_count(filter, tp, LOCATION_GRAVE, 0, None) as u32 * 300;
    duel.set_target_player(1 - tp);
    duel.set_target_param(dam);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, dam as i32);
    Ok(false)
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d = duel.get_matching_group_count(filter, a.tp, LOCATION_GRAVE, 0, None) as u32 * 300;
    duel.damage(p, d, REASON_EFFECT)?;
    Ok(())
}

fn checkop(duel: &mut Duel, a: &Args) -> Script<()> {
    if a.ep != a.e.get_handler(duel).get_controler(duel) {
        duel.set_script_state(STATE, 0);
    }
    Ok(())
}

fn clear(duel: &mut Duel, _: &Args) -> Script<()> {
    duel.set_script_state(STATE, 1);
    Ok(())
}
