// Abyss of the Sacred Beasts
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, MZONE, OPPONENT, SZONE};

const ID: u32 = 86132414;

pub const LISTED_NAMES: &[u32] = &[ID];

// Three copies on the field for the cost, more in the Deck and Graveyard, and a monster of the opponent's to turn face-down.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, 0),
    (ID, SZONE),
    (ID, SZONE),
    (ID, DECK),
    (ID, GRAVE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_POSITION);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_cost(duel, poscost);
    e1.set_target(duel, postg);
    e1.set_operation(duel, posop);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, (ID, 0));
    e2.set_condition(duel, |duel, _| duel.is_main_phase());
    e2.set_target(duel, pltg);
    e2.set_operation(duel, plop);
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_TODECK);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, |duel, a| {
        duel.get_current_phase() == PHASE_END && duel.is_turn_player(1 - a.tp)
    });
    e3.set_target(duel, tdtg);
    e3.set_operation(duel, tdop);
    e3.set_hint_timing(duel, 0, TIMING_END_PHASE);
    c.register_effect(duel, e3);
}

fn poscostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, ID) && c.is_able_to_grave_as_cost(duel) && c.is_faceup(duel)
}

fn poscost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_grave_as_cost(duel)
            && c.is_status(duel, STATUS_EFFECT_ENABLED)
            && duel.is_existing_matching_card(poscostfilter, tp, LOCATION_ONFIELD, 0, 2, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, poscostfilter, tp, LOCATION_ONFIELD, 0, 2, 2, Some(c.into()))?;
    duel.send_to_grave(g.union(c.into()), REASON_COST)?;
    Ok(false)
}

fn postg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE) && chkc.is_can_turn_set(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_can_turn_set, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g = duel.select_target(tp, CardId::is_can_turn_set, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_POSITION, g, 1, tp, POS_FACEDOWN_DEFENSE as i32);
    Ok(false)
}

fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && tc.is_monster(duel)
    {
        duel.change_position(tc, POS_FACEDOWN_DEFENSE)?;
    }
    Ok(())
}

fn selfplfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_code(duel, ID) && !c.is_forbidden(duel) && c.check_unique_on_field(duel, tp)
}

fn pltg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) >= 2
            && duel.is_existing_matching_card(
                |c, duel| selfplfilter(c, duel, tp),
                tp,
                LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE,
                0,
                2,
                None,
            ));
    }
    Ok(false)
}

fn plop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.get_location_count(tp, LOCATION_SZONE) < 2 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    let filter = aux::necro_valley_filter(|c, duel| selfplfilter(c, duel, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE, 0, 2, 2, None)?;
    if g.len() == 2 {
        for tc in g {
            duel.move_to_field(tc, tp, tp, LOCATION_SZONE, POS_FACEUP, true)?;
        }
    }
    Ok(())
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_deck(duel));
    }
    duel.set_operation_info(0, CATEGORY_TODECK, Some(c.into()), 1, a.tp, 0);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.send_to_deck(c, PLAYER_NONE, SEQ_DECKBOTTOM, REASON_EFFECT)?;
    }
    Ok(())
}
