// Skull Meister
use crate::prelude::*;
use crate::testing::{GRAVE, HAND, OPPONENT};

const ID: u32 = 67750322;

// Medius the Pure, whose effect the opponent activates in the GY.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (97556336, GRAVE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, discon);
    e1.set_cost(duel, Cost::self_to_grave);
    e1.set_target(duel, distg);
    e1.set_operation(duel, |duel, a| {
        duel.negate_effect(a.ev as u8);
        Ok(())
    });
    c.register_effect(duel, e1);
    let e6 = duel.create_effect(c);
    e6.set_type(duel, EFFECT_TYPE_SINGLE);
    e6.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_SINGLE_RANGE);
    e6.set_range(duel, LOCATION_HAND);
    e6.set_code(duel, 3682106);
    c.register_effect(duel, e6);
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let ev = a.ev as u8;
    let loc: u32 = duel.get_chain_info(ev, CHAININFO_TRIGGERING_LOCATION);
    a.ep != a.tp && duel.is_chain_disablable(ev) && loc == LOCATION_GRAVE
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(a.re.is_some_and(|re| !re.get_handler(duel).is_status(duel, STATUS_DISABLED)));
    }
    duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, 0, 0);
    Ok(false)
}
