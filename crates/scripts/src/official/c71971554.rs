// Road Synchron
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 71971554;

// Three on the field against the opponent's Elfnote Lucina to attack into.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (ID, MZONE), (13597785, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];
const ROAD_WARRIOR: u32 = 2322421;

pub const LISTED_NAMES: &[u32] = &[ROAD_WARRIOR];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EVENT_DAMAGE_STEP_END);
    e1.set_condition(duel, lvcon);
    e1.set_operation(duel, lvop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_SYNCHRO_LEVEL);
    e2.set_value(duel, Value::Card(lvval));
    c.register_effect(duel, e2);
}

fn lvcon(duel: &Duel, a: &Args) -> bool {
    duel.get_attacker() == Some(a.e.get_handler(duel))
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_LEVEL);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    Ok(())
}

fn lvval(duel: &Duel, e: EffectId, c: CardId) -> i32 {
    let lv = e.get_handler(duel).get_level(duel) as i32;
    if c.is_code(duel, ROAD_WARRIOR) {
        lv
    } else if lv <= 2 {
        16
    } else {
        lv - 2
    }
}
