// Shien's Squire
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 33883834;

pub const LISTED_SERIES: &[u16] = &[SET_SIX_SAMURAI];
// In the hand beside Six Samurai monsters on the field to battle the opponent's Elfnote Lucinas.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (2511717, MZONE),
    (2511717, MZONE),
    (44686185, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_code(duel, EVENT_PRE_DAMAGE_CALCULATE);
    e1.set_condition(duel, con);
    e1.set_cost(duel, cost);
    e1.set_operation(duel, op);
    c.register_effect(duel, e1);
}

fn con(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    let (Some(at), Some(d)) = (duel.get_attacker(), duel.get_attack_target()) else {
        return false;
    };
    ((at.is_controler(duel, tp) && at.is_set_card(duel, &[SET_SIX_SAMURAI]))
        || (d.is_controler(duel, tp) && d.is_set_card(duel, &[SET_SIX_SAMURAI])))
        && duel.get_flag_effect(tp, ID) == 0
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_grave_as_cost(duel));
    }
    duel.send_to_grave(c, REASON_COST)?;
    duel.register_flag_effect(a.tp, ID, RESET_PHASE | PHASE_DAMAGE_CAL, 0, 1);
    Ok(false)
}

fn op(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let (at, d) = (duel.get_attacker(), duel.get_attack_target());
    unported!(duel, at.is_none() || d.is_none());
    let (Some(at), Some(d)) = (at, d) else { return Ok(()) };
    if !at.is_relate_to_battle(duel) || !d.is_relate_to_battle(duel) {
        return Ok(());
    }
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_description(duel, 3000);
    e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
    e1.set_owner_player(duel, tp);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    e1.set_value(duel, 1);
    if at.is_controler(duel, tp) {
        at.register_effect(duel, e1);
    } else {
        d.register_effect(duel, e1);
    }
    Ok(())
}
