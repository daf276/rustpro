// Live☆Twin Lil-la Treat
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 81078880;

// In the hand and Graveyard beside Live☆Twin Ki-sikil Frost (a Ki-sikil monster for the hand Special Summon) and Evil★Twin Lil-la (for the
// battle damage), with an attacker for the opponent.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (54257392, MZONE),
    (36609518, MZONE),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, spcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_BATTLE_DAMAGE);
    e2.set_category(duel, CATEGORY_ATKCHANGE);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, condition);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    duel.get_location_count(c.get_controler(duel), LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_KI_SIKIL])),
            a.e.get_handler_player(duel),
            LOCATION_MZONE,
            0,
            1,
            None,
        )
}

// The Lua's `or Duel.GetBattleDamage(1-tp)` is a number, always true.
fn condition(duel: &Duel, _: &Args) -> bool {
    let attacker = duel.get_attacker();
    unported!(duel, attacker.is_none());
    let evil_twin = |c: Option<CardId>| c.is_some_and(|c| c.is_set_card(duel, &[SET_EVIL_TWIN]));
    evil_twin(attacker) || evil_twin(duel.get_attack_target())
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_faceup, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, CardId::is_faceup, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_faceup(duel)
        && tc.is_relate_to_effect(duel, e)
    {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        let damage = if duel.get_battle_damage(tp) > 0 {
            duel.get_battle_damage(tp)
        } else {
            duel.get_battle_damage(1 - tp)
        };
        e1.set_value(duel, -damage);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
