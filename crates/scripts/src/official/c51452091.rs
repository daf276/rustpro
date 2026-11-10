// Royal Decree
use crate::prelude::*;
use crate::testing::{HAND, OPPONENT, SZONE};

const ID: u32 = 51452091;

// In the hand, with Mirror Force of the opponent's to be negated.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (44095762, SZONE | OPPONENT), (44095762, SZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_END_PHASE);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_DISABLE);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_target_range(duel, LOCATION_SZONE, LOCATION_SZONE);
    e2.set_target_card(duel, distarget);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EVENT_CHAIN_SOLVING);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_operation(duel, disop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD);
    e4.set_code(duel, EFFECT_DISABLE_TRAPMONSTER);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e4.set_target_card(duel, distarget);
    c.register_effect(duel, e4);
    aux::double_snare_validity(duel, c, LOCATION_SZONE);
}

fn distarget(duel: &Duel, e: EffectId, c: CardId) -> bool {
    c != e.get_handler(duel) && c.is_trap(duel)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, ev) = (a.e, a.ev as u8);
    let tl: u32 = duel.get_chain_info(ev, CHAININFO_TRIGGERING_LOCATION);
    if tl == LOCATION_SZONE
        && let Some(re) = a.re
        && re.is_trap_effect(duel)
        && re.get_handler(duel) != e.get_handler(duel)
    {
        duel.negate_effect(ev);
    }
    Ok(())
}
