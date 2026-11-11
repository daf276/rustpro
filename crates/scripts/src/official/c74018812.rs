// Labrynth Stovie Torbie
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, SZONE};

const ID: u32 = 74018812;

// Twice in the hand, one to send and one to discard, and in the Graveyard, with Welcome Labrynth in the Deck to Set and
// Normal Traps (Mirror Force, Trap Hole) Set to make a monster leave the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (5380979, DECK),
    (5380979, DECK),
    (44095762, SZONE),
    (4206964, SZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_HAND | LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_hint_timing(duel, TIMING_END_PHASE, TIMINGS_CHECK_MONSTER_E);
    e1.set_cost(duel, setcost);
    e1.set_operation(duel, setop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property_full(duel, EFFECT_FLAG_DELAY, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e2.set_code(duel, EVENT_LEAVE_FIELD);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spcon);
    e2.set_target(duel, stock::special_summon_self_target_player_0);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e2);
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_LABRYNTH]) && c.is_spell_trap(duel) && c.is_ssetable(duel)
}

fn setcostfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_discardable(duel) && duel.is_existing_matching_card(setfilter, tp, LOCATION_HAND | LOCATION_DECK, 0, 1, Some(c.into()))
}

fn setcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    if chk == 0 {
        return Ok(c.is_able_to_grave_as_cost(duel)
            && duel.is_existing_matching_card(|g, duel| setcostfilter(g, duel, tp), tp, LOCATION_HAND, 0, 1, Some(c.into())));
    }
    duel.send_to_grave(c, REASON_COST)?;
    duel.discard_hand(tp, |g, duel| setcostfilter(g, duel, tp), 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let g = duel.select_matching_card(tp, setfilter, tp, LOCATION_HAND | LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.sset(tp, g)?;
    }
    Ok(())
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_previous_location(duel, LOCATION_MZONE) && c.is_reason(duel, REASON_EFFECT)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    a.rp == a.tp
        && a.re
            .is_some_and(|re| re.is_trap_effect(duel) && re.get_handler(duel).get_original_type(duel) == TYPE_TRAP)
        && a.eg.is_some_and(|eg| eg.is_exists(duel, cfilter, 1, None) && !eg.contains(c))
}
