// Deck Lockdown
use crate::prelude::*;
use crate::testing::{HAND, SZONE};

const ID: u32 = 1149109;

// One in the hand to activate, and a second to count the turns of.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, SZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    c.register_effect(duel, e1);
    for code in [EFFECT_CANNOT_TO_HAND, EFFECT_CANNOT_SPECIAL_SUMMON] {
        let e = duel.create_effect(c);
        e.set_type(duel, EFFECT_TYPE_FIELD);
        e.set_range(duel, LOCATION_SZONE);
        e.set_code(duel, code);
        e.set_target_range(duel, LOCATION_DECK, LOCATION_DECK);
        c.register_effect(duel, e);
    }
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let c = a.e.get_handler(duel);
    c.set_turn_counter(duel, 0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e1.set_operation(duel, desop);
    e1.set_reset_full(
        duel,
        RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_STANDBY | RESET_SELF_TURN,
        2,
    );
    c.register_effect(duel, e1);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let ct = c.get_turn_counter(duel) + 1;
    c.set_turn_counter(duel, ct);
    if ct == 2 {
        duel.destroy(c, REASON_RULE)?;
    }
    Ok(())
}
