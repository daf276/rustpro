// Medium of the Ice Barrier
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 88494899;

// The opponent's field filled with Elfnote Lucinas and Mystical Space Typhoons so it controls four more cards.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (5318639, SZONE | OPPONENT),
    (5318639, SZONE | OPPONENT),
    (5318639, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, spcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_FIELD);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_operation(duel, aclimit1);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_FIELD);
    e3.set_code(duel, EVENT_CHAIN_NEGATED);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_operation(duel, aclimit2);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD);
    e4.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e4.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_target_range(duel, 0, 1);
    e4.set_condition(duel, |duel, a| a.e.get_handler(duel).get_flag_effect(duel, ID) != 0);
    e4.set_value(
        duel,
        Value::EffectPlayer(|duel, _, te, _| te.is_has_type(duel, EFFECT_TYPE_ACTIVATE)),
    );
    c.register_effect(duel, e4);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else {
        return true;
    };
    let tp = c.get_controler(duel);
    duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.get_field_group_count(tp, 0, LOCATION_ONFIELD) as i64 - duel.get_field_group_count(tp, LOCATION_ONFIELD, 0) as i64 >= 4
}

fn opponent_activated(duel: &Duel, a: &Args) -> bool {
    a.ep != a.tp && a.re.is_some_and(|re| re.is_has_type(duel, EFFECT_TYPE_ACTIVATE))
}

fn aclimit1(duel: &mut Duel, a: &Args) -> Script<()> {
    if opponent_activated(duel, a) {
        let reset = RESET_EVENT | RESETS_STANDARD_DISABLE | RESET_CONTROL | RESET_PHASE | PHASE_END;
        a.e.get_handler(duel).register_flag_effect(duel, ID, reset, 0, 1);
    }
    Ok(())
}

fn aclimit2(duel: &mut Duel, a: &Args) -> Script<()> {
    if opponent_activated(duel, a) {
        a.e.get_handler(duel).reset_flag_effect(duel, ID);
    }
    Ok(())
}
