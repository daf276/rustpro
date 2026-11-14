// Soul Drain
use crate::prelude::*;
use crate::testing::{HAND, SZONE};

const ID: u32 = 73599290;

pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE), (ID, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_cost(duel, Cost::pay_lp::<1000>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_target_range(duel, 1, 1);
    e2.set_value(duel, Value::EffectPlayer(aclimit));
    c.register_effect(duel, e2);
}

fn aclimit(duel: &Duel, _: EffectId, re: EffectId, _: u8) -> bool {
    let loc = re.get_activate_location(duel);
    (loc == LOCATION_GRAVE || loc == LOCATION_REMOVED) && re.is_monster_effect(duel)
}
