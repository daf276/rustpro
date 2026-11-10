// Flying Elephant
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 66765023;

// On the field, to survive the opponent's Raigeki; the win itself is no check, as every duel ends with MSG_WIN.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (12580477, HAND | OPPONENT), (12580477, HAND | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_DESTROY_REPLACE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target(duel, reptg);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_PHASE + PHASE_END);
    e2.set_count_limit(duel, 1, 0);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).has_flag_effect(duel, ID));
    e2.set_operation(duel, effop);
    c.register_effect(duel, e2);
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.is_reason(duel, REASON_EFFECT)
            && c.get_reason_player(duel) != tp
            && duel.is_turn_player(1 - tp)
            && !c.has_flag_effect(duel, ID));
    }
    c.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
    Ok(true)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_BATTLE_DAMAGE);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END | RESET_SELF_TURN);
    e1.set_condition(duel, |duel, a| a.ep != a.tp && duel.get_attack_target().is_none());
    e1.set_operation(duel, |duel, a| {
        duel.win(a.tp, WIN_REASON_FLYING_ELEPHANT);
        Ok(())
    });
    c.register_effect(duel, e1);
    Ok(())
}
