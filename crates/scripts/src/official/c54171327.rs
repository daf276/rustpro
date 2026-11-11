// Angelechy Disturbance
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, MZONE, OPPONENT, SZONE};

const ID: u32 = 54171327;

pub const LISTED_NAMES: &[u32] = &[ID];

// Set, with an Angelechy monster on the field to give away next to the opponent's cards to negate, and a copy in the Graveyard
// to banish for an Angelechy Spell/Trap in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, GRAVE),
    (17782288, DECK),
    (17782288, DECK),
    (81797573, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (44095762, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_CONTROL + CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, ctrltg);
    e1.set_operation(duel, ctrlop);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e2);
}

fn ctrlfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ANGELECHY]) && c.is_controler_can_be_changed(duel) && c.is_faceup(duel)
}

fn ctrltg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(ctrlfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_CONTROL, None, 1, tp, LOCATION_MZONE as i32);
    duel.set_possible_operation_info(0, CATEGORY_DISABLE, None, 1, 1 - tp, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn ctrlop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONTROL);
    let g = duel.select_matching_card(tp, ctrlfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    // The Lua tests the returned count, and 0 is truthy there.
    duel.get_control_full(tc, 1 - tp, PHASE_END as u16, 1, 0xff, duel.get_reason_player())?;
    if !tc.is_controler(duel, 1 - tp) {
        return Ok(());
    }
    let c = a.e.get_handler(duel);
    let seq = tc.get_sequence(duel);
    let up = match seq {
        1 => Some(5),
        3 => Some(6),
        _ => None,
    };
    let zones = [
        (LOCATION_SZONE, Some(seq)),
        (LOCATION_MZONE, seq.checked_sub(1)),
        (LOCATION_MZONE, (seq < 4).then_some(seq + 1)),
        (LOCATION_MZONE, up),
    ];
    let mut break_chk = false;
    for (loc, nseq) in zones {
        let Some(nc) = nseq.and_then(|nseq| duel.get_field_card(1 - tp, loc, nseq)) else {
            continue;
        };
        if nc.is_negatable(duel) {
            if !break_chk {
                break_chk = true;
                duel.break_effect()?;
            }
            nc.negate_effects(duel, c, 0, false);
        }
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ANGELECHY]) && c.is_spell_trap(duel) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
