// Constellar Omega
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE};

const ID: u32 = 26329679;

pub const LISTED_SERIES: &[u16] = &[SET_CONSTELLAR];
// In the Extra Deck, Xyz Summoned with two Level 4 LIGHT monsters (Mystical Elf, Ancient Elf).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (15025844, MZONE),
    (15025844, MZONE),
    (93221206, MZONE),
    (93221206, MZONE),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_attribute_full(duel, ATTRIBUTE_LIGHT, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_operation(duel, operation);
    e1.set_hint_timing(
        duel,
        TIMING_STANDBY_PHASE | TIMING_END_PHASE,
        TIMING_STANDBY_PHASE | TIMING_END_PHASE,
    );
    c.register_effect(duel, e1);
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(
        aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_CONSTELLAR])),
        a.tp,
        LOCATION_MZONE,
        0,
        None,
    );
    for tc in g.iter() {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_description(duel, 3104);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_range(duel, LOCATION_MZONE);
        e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        e1.set_value(duel, Value::Effect(|duel, _, te| te.is_spell_trap_effect(duel)));
        tc.register_effect(duel, e1);
    }
    Ok(())
}
