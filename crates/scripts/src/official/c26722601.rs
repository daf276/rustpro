// Steelswarm Gatekeeper
use engine::constants::MSG_HINT;

use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 26722601;

pub const LISTED_SERIES: &[u16] = &[SET_STEELSWARM];
// On the field, with Steelswarm Moth in the hand to Tribute Summon with Steelswarm Sting, and cards to summon or set again.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, HAND),
    (22009013, HAND),
    (22009013, HAND),
    (22009013, HAND),
    (35618486, MZONE),
    (35618486, MZONE),
];
// The only effect is continuous and forms no chain link: the extra summon it grants shows only as a card hint.
pub const CHECKS: &[Check] = &[Check::Report(&Check::Message(MSG_HINT))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_operation(duel, sumop);
    c.register_effect(duel, e1);
}

fn sumop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(ec) = a.eg.and_then(|eg| eg.get_first()) else {
        return Ok(());
    };
    if ec != c && ec.is_set_card(duel, &[SET_STEELSWARM]) && ec.is_tribute_summoned(duel) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_range(duel, LOCATION_MZONE);
        e1.set_description(duel, aux::stringid(ID, 0));
        e1.set_target_range(duel, LOCATION_HAND | LOCATION_MZONE, 0);
        e1.set_code(duel, EFFECT_EXTRA_SUMMON_COUNT);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_EXTRA_SET_COUNT);
        c.register_effect(duel, e2);
    }
    Ok(())
}
