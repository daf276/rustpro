// Mulcharmy Meowls
use super::c42141493::{add_mulcharmy_counter, drop, effcost, efftg, register_shuffle};
use crate::prelude::*;

const ID: u32 = 87126721;

pub const LISTED_SERIES: &[u16] = &[SET_MULCHARMY];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DRAW + CATEGORY_TODECK);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    e1.set_condition(duel, |duel, a| duel.get_field_group_count(a.tp, LOCATION_ONFIELD, 0) == 0);
    e1.set_cost(duel, effcost::<ID>);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
    add_mulcharmy_counter(duel, ID);
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, drcon);
    e1.set_operation(duel, drop::<ID>);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, a.tp);
    register_shuffle::<ID>(duel, c, a.tp);
    Ok(())
}

fn drconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_summon_player(duel, 1 - tp) && c.is_summon_location(duel, LOCATION_GRAVE | LOCATION_REMOVED)
}

fn drcon(duel: &Duel, a: &Args) -> bool {
    a.eg.unwrap_or_default()
        .is_exists(duel, |c, duel| drconfilter(c, duel, a.tp), 1, None)
}
