// Kumongous, the Sticky String Kaiju
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 29726552;

pub const COUNTER_LIST: &[u16] = &[COUNTER_KAIJU];

// In the hand with a copy on the opponent's field to Tribute, Kyoutou Waterfront for Kaiju Counters, Raigeki and Dark Hole to fill it, and
// the opponent's Sangan to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE | OPPONENT),
    (56111151, HAND),
    (12580477, HAND),
    (53129443, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, HAND | OPPONENT),
    (26202165, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    aux::add_kaiju_procedure(duel, c);
    let e3 = duel.create_effect(c);
    e3.set_category(duel, CATEGORY_DISABLE);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_SUMMON_SUCCESS);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_cost(duel, aux::kaiju_counter_cost::<2>);
    e3.set_target(duel, target);
    e3.set_operation(duel, operation);
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e4);
}

fn filter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_summon_player(duel, tp) && c.is_faceup(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp, eg) = (a.e.get_handler(duel), a.tp, a.eg.unwrap_or_default());
    if chk == 0 {
        return Ok(eg.is_exists(duel, |c, duel| filter(c, duel, 1 - tp), 1, None) && !eg.contains(c));
    }
    let g = eg.filter(duel, |c, duel| filter(c, duel, 1 - tp), None);
    duel.set_target_card(g);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    let g = duel.get_target_cards(e).unwrap_or_default();
    for tc in g.iter() {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3206);
        e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CANNOT_ATTACK);
        e1.set_reset_full(duel, RESETS_STANDARD_PHASE_END, 2);
        tc.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_code(duel, EFFECT_DISABLE);
        e2.set_reset_full(duel, RESETS_STANDARD_PHASE_END, 2);
        tc.register_effect(duel, e2);
        let e3 = duel.create_effect(c);
        e3.set_type(duel, EFFECT_TYPE_SINGLE);
        e3.set_code(duel, EFFECT_DISABLE_EFFECT);
        e3.set_reset_full(duel, RESETS_STANDARD_PHASE_END, 2);
        tc.register_effect(duel, e3);
    }
    Ok(())
}
