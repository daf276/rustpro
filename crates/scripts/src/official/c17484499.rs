// Exchange of the Spirit
use crate::prelude::*;
use crate::testing::{Check, HAND};

const ID: u32 = 17484499;

// Random play often fills both Graveyards with 15 cards.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_DUEL);
    e1.set_condition(duel, |duel, a| {
        duel.get_field_group_count(a.tp, LOCATION_GRAVE, 0) >= 15 && duel.get_field_group_count(a.tp, 0, LOCATION_GRAVE) >= 15
    });
    e1.set_cost(duel, Cost::pay_lp::<1000>);
    e1.set_operation(duel, |duel, _| {
        let turn_player = duel.get_turn_player();
        duel.swap_deck_and_grave(turn_player);
        duel.swap_deck_and_grave(1 - turn_player);
        Ok(())
    });
    e1.set_hint_timing(
        duel,
        0,
        TIMING_DRAW_PHASE
            | TIMING_STANDBY_PHASE
            | TIMING_MAIN_END
            | TIMING_BATTLE_START
            | TIMING_BATTLE_END
            | TIMINGS_CHECK_MONSTER_E
            | TIMING_CHAIN_END,
    );
    c.register_effect(duel, e1);
}
