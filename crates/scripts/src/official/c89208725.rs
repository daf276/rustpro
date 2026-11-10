// Metaverse
use crate::prelude::*;
use crate::testing::{DECK, SZONE};

const ID: u32 = 89208725;

// Set, with Field Spells (Umi, Mountain, Wasteland) in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE), (22702055, DECK), (50913601, DECK), (23424603, DECK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn thactfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_field_spell(duel)
        && (c.is_able_to_hand(duel)
            || c.get_activate_effect(duel)
                .is_some_and(|te| te.is_activatable(duel, tp, true, true)))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| thactfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    if !duel.check_phase_activity() {
        duel.register_flag_effect(tp, CARD_MAGICAL_MIDBREAKER, RESET_CHAIN, 0, 1);
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 1));
    let filter = |c, duel: &Duel| thactfilter(c, duel, tp);
    let Some(sc) = duel.select_matching_card(tp, filter, tp, LOCATION_DECK, 0, 1, 1, None)?.get_first() else {
        return Ok(());
    };
    aux::to_hand_or_else(
        duel,
        sc,
        tp,
        |sc, duel| {
            sc.get_activate_effect(duel)
                .is_some_and(|te| te.is_activatable(duel, tp, true, true))
        },
        |duel| duel.activate_field_spell(sc, tp).map(drop),
        aux::stringid(ID, 2),
    )
}
