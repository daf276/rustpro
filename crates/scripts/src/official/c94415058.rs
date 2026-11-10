// Stargazer Magician
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, PZONE, SZONE};

const ID: u32 = 94415058;

// Copies in the Pendulum Zone, on the field and in the hand (to attack with and to be summoned after Compulsory Evacuation
// Device, set or in the opponent's hand, bounces one).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, HAND),
    (94192409, SZONE | OPPONENT),
    (94192409, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK), Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e2.set_range(duel, LOCATION_PZONE);
    e2.set_condition(duel, actcon);
    e2.set_operation(duel, actop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_BE_BATTLE_TARGET);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE);
    e4.set_code(duel, EFFECT_CHANGE_LSCALE);
    e4.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e4.set_range(duel, LOCATION_PZONE);
    e4.set_condition(duel, sccon);
    e4.set_value(duel, 4);
    c.register_effect(duel, e4);
    let e5 = e4.clone_effect(duel);
    e5.set_code(duel, EFFECT_CHANGE_RSCALE);
    c.register_effect(duel, e5);
    let e6 = duel.create_effect(c);
    e6.set_description(duel, aux::stringid(ID, 0));
    e6.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e6.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e6.set_code(duel, EVENT_TO_HAND);
    e6.set_range(duel, LOCATION_MZONE);
    e6.set_count_limit(duel, 1, 0);
    e6.set_condition_mut(duel, spcon);
    e6.set_target(duel, sptg);
    e6.set_operation(duel, spop);
    c.register_effect(duel, e6);
}

fn actcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    let Some(mut tc) = duel.get_attacker() else {
        return false;
    };
    if tc.is_controler(duel, 1 - tp) {
        let Some(target) = duel.get_attack_target() else {
            return false;
        };
        tc = target;
    }
    tc.is_controler(duel, tp) && tc.is_type(duel, TYPE_PENDULUM)
}

fn actop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_target_range(duel, 0, 1);
    e1.set_value(
        duel,
        Value::EffectPlayer(|duel, _, re, _| re.is_spell_effect(duel) && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)),
    );
    e1.set_reset(duel, RESET_PHASE | PHASE_DAMAGE);
    duel.register_effect(e1, a.tp);
    Ok(())
}

fn sccon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    !duel.is_existing_matching_card(
        |c, duel| c.is_set_card(duel, &[SET_MAGICIAN, SET_ODD_EYES]),
        a.e.get_handler_player(duel),
        LOCATION_PZONE,
        0,
        1,
        Some(c.into()),
    )
}

fn spcon(duel: &mut Duel, a: &Args) -> bool {
    let eg = a.eg.unwrap_or_default();
    let Some(tc) = eg.get_first() else {
        return false;
    };
    let tp = a.tp;
    a.e.set_label(duel, i64::from(tc.get_code(duel)));
    eg.len() == 1
        && a.rp != tp
        && tc.is_reason(duel, REASON_EFFECT)
        && tc.is_previous_controler(duel, tp)
        && tc.is_previous_location(duel, LOCATION_MZONE)
        && tc.is_type(duel, TYPE_PENDULUM)
        && tc.is_controler(duel, tp)
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8, code: u32) -> bool {
    c.is_code(duel, code) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let code = e.get_label(duel) as u32;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| filter(c, duel, e, tp, code), tp, LOCATION_HAND, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    let code = e.get_label(duel) as u32;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| filter(c, duel, e, tp, code), tp, LOCATION_HAND, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
