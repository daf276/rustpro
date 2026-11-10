// Necroface
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 28297833;

// In the hand to Normal Summon and on the field to attack, banished by the opponent's Dimensional Prisons, and a Sangan to attack.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (70342110, SZONE | OPPONENT),
    (70342110, SZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TODECK + CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_target(duel, tdtg);
    e1.set_operation(duel, tdop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_REMOVE);
    e2.set_target(duel, rmtg);
    e2.set_operation(duel, rmop);
    c.register_effect(duel, e2);
}

fn tdtg(duel: &mut Duel, _: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_TODECK, None, 1, PLAYER_ALL, LOCATION_REMOVED as i32);
    Ok(false)
}

fn register_attack(duel: &mut Duel, c: CardId, value: i32, reset: u32) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_reset(duel, reset);
    e1.set_value(duel, value);
    c.register_effect(duel, e1);
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let g = duel.get_field_group(a.tp, LOCATION_REMOVED, LOCATION_REMOVED);
    duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    let ct = g.filter_count(duel, |c, duel| c.is_location(duel, LOCATION_DECK), None);
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, a.e) {
        register_attack(duel, c, ct as i32 * 100, RESET_EVENT | RESETS_STANDARD_DISABLE);
    }
    Ok(())
}

fn rmtg(duel: &mut Duel, _: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 5, PLAYER_ALL, LOCATION_DECK as i32);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let mut g1 = duel.get_decktop_group(tp, 5);
    g1.merge(duel.get_decktop_group(1 - tp, 5));
    duel.disable_shuffle_check(true);
    duel.remove(g1, POS_FACEUP, REASON_EFFECT)?;
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, a.e) {
        register_attack(duel, c, 500, RESETS_STANDARD_PHASE_END);
    }
    Ok(())
}
