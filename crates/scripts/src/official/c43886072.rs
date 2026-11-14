// Prank-Kids Bow-Wow-Bark
use super::c17382973::thtg2;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 43886072;

pub const LISTED_SERIES: &[u16] = &[SET_PRANK_KIDS];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_PRANK_KIDS], Some(lc), sumtype, tp)
    }
}

// Prank-Kids Dropsies, Fansies and Lampsies to Link Summon with and to add from the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (55725117, MZONE),
    (81119816, MZONE),
    (18236002, MZONE),
    (55725117, MZONE),
    (55725117, GRAVE),
    (81119816, GRAVE),
    (18236002, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1.set_target_card(duel, atktg);
    e1.set_value(duel, 1000);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E | TIMING_MAIN_END);
    e2.set_condition(duel, |duel, a| !duel.is_turn_player(a.tp));
    e2.set_cost(duel, |duel, a, chk| Cost::replaceable(duel, a, chk, Cost::self_tribute));
    e2.set_target(duel, thtg2);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn atktg(duel: &Duel, e: EffectId, c: CardId) -> bool {
    e.get_handler(duel).get_linked_group(duel).is_contains(c) && c.is_set_card(duel, &[SET_PRANK_KIDS])
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_target_cards(e).unwrap_or_default();
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    e1.set_target_card(duel, indtg);
    e1.set_value(duel, Value::EffectPlayer(aux::indoval));
    duel.register_effect(e1, tp);
    Ok(())
}

fn indtg(duel: &Duel, _: EffectId, c: CardId) -> bool {
    c.is_set_card(duel, &[SET_PRANK_KIDS])
}
