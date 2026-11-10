// Gear Golem the Moving Fortress
use crate::prelude::*;
use crate::testing::{HAND, MZONE};

const ID: u32 = 30190809;

// On the field, and in the hand to be Normal Summoned.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| {
        duel.is_phase(PHASE_MAIN1) && a.e.get_handler(duel).get_effect_count(duel, EFFECT_DIRECT_ATTACK) == 0
    });
    e1.set_cost(duel, Cost::pay_lp::<800>);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_DIRECT_ATTACK);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    c.register_effect(duel, e1);
    Ok(())
}
