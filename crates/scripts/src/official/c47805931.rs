// Number 20: Giga-Brilliant
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE};

const ID: u32 = 47805931;

// In the Extra Deck, Xyz Summoned with two Level 3 monsters.
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (32864, MZONE), (1761063, MZONE), (2468169, MZONE)];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(3);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let handler = a.e.get_handler(duel);
    let g = duel.get_matching_group(CardId::is_faceup, a.tp, LOCATION_MZONE, 0, None);
    for tc in g {
        let e1 = duel.create_effect(handler);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_value(duel, 300);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
