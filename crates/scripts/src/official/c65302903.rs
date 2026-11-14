// Dominusteer Rosepamela
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 65302903;

pub const LISTED_SERIES: &[u16] = &[SET_DOMINUS_PURGE, SET_DOMINATORS];
// Four copies in the hand to Normal Summon and reveal on the opponent's turn, and one on the field, with Dominus Impulse and Dominus
// Purge in the Deck to search or Set.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (40366667, DECK),
    (40366667, DECK),
    (97045737, DECK),
    (97045737, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_STZONE, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_facedown(duel));
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_HANDES);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, (ID, 0));
    e2.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e2.set_cost(duel, Cost::self_reveal);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e2);
    let e3a = duel.create_effect(c);
    e3a.set_description(duel, aux::stringid(ID, 1));
    e3a.set_category(duel, CATEGORY_SET);
    e3a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3a.set_property(duel, EFFECT_FLAG_DELAY);
    e3a.set_code(duel, EVENT_SUMMON_SUCCESS);
    e3a.set_count_limit(duel, 1, (ID, 1));
    e3a.set_target(duel, settg);
    e3a.set_operation(duel, setop);
    c.register_effect(duel, e3a);
    let e3b = e3a.clone_effect(duel);
    e3b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3b);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DOMINUS_PURGE]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, tp, 1);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(sc) = g.first()
        && duel.send_to_hand(sc, PLAYER_NONE, REASON_EFFECT)? > 0
        && sc.is_location(duel, LOCATION_HAND)
    {
        duel.confirm_cards(1 - tp, sc)?;
        duel.shuffle_hand(tp)?;
        duel.shuffle_deck(tp)?;
        duel.break_effect()?;
        duel.discard_hand_full(
            tp,
            |c, duel| c.is_discardable_full(duel, REASON_EFFECT, None),
            1,
            1,
            REASON_EFFECT | REASON_DISCARD,
            None,
        )?;
    }
    Ok(())
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DOMINATORS, SET_DOMINUS_PURGE]) && c.is_trap(duel) && c.is_ssetable(duel)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, setfilter, LOCATION_DECK | LOCATION_GRAVE, CATEGORY_SET)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(setfilter),
        tp,
        LOCATION_DECK | LOCATION_GRAVE,
        0,
        1,
        1,
        None,
    )?;
    if let Some(sc) = g.first()
        && duel.sset(tp, sc)? > 0
    {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_description(duel, aux::stringid(ID, 2));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_SET_AVAILABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_TRAP_ACT_IN_SET_TURN);
        e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_facedown(duel));
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        sc.register_effect(duel, e1);
    }
    Ok(())
}
