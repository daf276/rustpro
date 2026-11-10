// Dark Ruler Ha Des
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};
use engine::constants::MSG_ATTACK;

const ID: u32 = 53982768;

// On the field to attack Sangan.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (26202165, MZONE | OPPONENT), (26202165, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_SINGLE_RANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_BATTLED);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let (attacker, target) = (duel.get_attacker(), duel.get_attack_target());
    let Some(d) = target else { return Ok(()) };
    // Lua errors on a nil attacker
    unported!(duel, attacker.is_none());
    let Some(at) = attacker else { return Ok(()) };
    let p = c.get_controler(duel);
    let tc = if at.is_controler(duel, p) && at.is_race(duel, RACE_FIEND) && d.is_status(duel, STATUS_BATTLE_DESTROYED) {
        d
    } else if d.is_controler(duel, p) && d.is_race(duel, RACE_FIEND) && at.is_status(duel, STATUS_BATTLE_DESTROYED) {
        at
    } else {
        return Ok(());
    };
    for code in [EFFECT_DISABLE, EFFECT_DISABLE_EFFECT] {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, code);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_EXC_GRAVE);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
