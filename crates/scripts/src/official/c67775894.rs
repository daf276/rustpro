// Wonder Wand
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 67775894;

struct Procedure;

impl EquipProcedure for Procedure {
    fn f(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_race(duel, RACE_SPELLCASTER)
    }
}

// In the hand, with Thousand-Eyes Idol and Lucky Trinket (Spellcasters) to equip to.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (27125110, MZONE), (27125110, MZONE), (3985011, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Equip::add_procedure::<Procedure>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_EQUIP);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_value(duel, 500);
    c.register_effect(duel, e2);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 0));
    e4.set_category(duel, CATEGORY_DRAW);
    e4.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_cost(duel, drcost);
    e4.set_target(duel, stock::draw_target::<2>);
    e4.set_operation(duel, stock::draw);
    c.register_effect(duel, e4);
}

fn drcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let c = a.e.get_handler(duel);
    let ec = c.get_equip_target(duel);
    if chk == 0 {
        return Ok(c.is_able_to_grave_as_cost(duel)
            && ec.is_some_and(|ec| c.get_controler(duel) == ec.get_controler(duel) && ec.is_able_to_grave_as_cost(duel)));
    }
    let mut g = CardSet::from(c);
    if let Some(ec) = ec {
        g.insert(ec);
    }
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}
