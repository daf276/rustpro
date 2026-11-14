// Mischief of the Gnomes
use crate::prelude::*;
use crate::testing::GRAVE;

const ID: u32 = 164710;

pub const SUPPORT: &[(u32, u16)] = &[(ID, GRAVE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER);
    e1.set_operation(duel, lvop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_operation(duel, lvop);
    c.register_effect(duel, e2);
}

fn lower_levels(duel: &mut Duel, c: CardId, hg: CardSet) {
    for tc in hg {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_LEVEL);
        e1.set_value(duel, -1);
        e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD_PHASE_END & !RESET_TOFIELD));
        tc.register_effect(duel, e1);
    }
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let hg = duel.get_matching_group(|c, duel| c.is_level_above(duel, 1), a.tp, LOCATION_HAND, LOCATION_HAND, None);
    lower_levels(duel, c, hg);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_TO_HAND);
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    e2.set_operation(duel, |duel, a| {
        let hg = a.eg.unwrap_or_default().filter(duel, |c, duel| c.is_level_above(duel, 1), None);
        lower_levels(duel, a.e.get_handler(duel), hg);
        Ok(())
    });
    duel.register_effect(e2, a.tp);
    Ok(())
}
