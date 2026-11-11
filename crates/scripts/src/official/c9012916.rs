// Black-Winged Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};
use engine::constants::MSG_ADD_COUNTER;

const ID: u32 = 9012916;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// In the Extra Deck, Synchro Summoned with Blackwing - Gale the Whirlwind (a Level 3 Tuner) and Blackwing - Sirocco the Dawn (Level 5), with Hinotama
// (46130346) in the opponent's hand to inflict effect damage.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (2009101, MZONE),
    (2009101, MZONE),
    (75498415, MZONE),
    (75498415, MZONE),
    (46130346, HAND | OPPONENT),
    (46130346, HAND | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::Message(MSG_ADD_COUNTER),
    Check::ResolvesEffect(aux::stringid(ID, 0)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_counter_permit(duel, COUNTER_FEATHER);
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CHANGE_DAMAGE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 1, 0);
    e1.set_value(duel, Value::ChangeDamage(damval));
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_NO_EFFECT_DAMAGE);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_code(duel, EFFECT_UPDATE_ATTACK);
    e3.set_value(
        duel,
        Value::Card(|duel, _, c| i32::from(c.get_counter(duel, COUNTER_FEATHER)) * -700),
    );
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 0));
    e4.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DAMAGE);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, 0);
    e4.set_cost(duel, cost);
    e4.set_target(duel, target);
    e4.set_operation(duel, operation);
    c.register_effect(duel, e4);
}

fn damval(duel: &mut Duel, e: EffectId, _: Option<EffectId>, val: u32, r: u32, _: u8, _: Option<CardId>) -> u32 {
    if r & REASON_EFFECT != 0 {
        e.get_handler(duel).add_counter(duel, COUNTER_FEATHER, 1);
        return 0;
    }
    val
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.get_counter(duel, COUNTER_FEATHER) > 0);
    }
    let ct = c.get_counter(duel, COUNTER_FEATHER);
    a.e.set_label(duel, i64::from(ct) * 700);
    c.remove_counter(duel, tp, COUNTER_FEATHER, ct, REASON_COST)?;
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::has_non_zero_attack, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATKDEF);
    let tc = duel.select_target(tp, CardId::has_non_zero_attack, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    let label = a.e.get_label(duel) as i32;
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, label);
    duel.set_operation_info(0, CATEGORY_ATKCHANGE, tc, 1, 0, -label);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && tc.is_faceup(duel)
        && !tc.is_immune_to_effect(duel, e)
    {
        let val = e.get_label(duel) as i32;
        let atk = tc.get_attack(duel);
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, -val);
        tc.register_effect(duel, e1);
        duel.damage(1 - tp, val.min(atk).max(0) as u32, REASON_EFFECT)?;
    }
    Ok(())
}
