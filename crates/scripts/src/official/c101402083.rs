// Just Flip a Coin
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 101402083;
const CHOICE: u8 = 0;

pub const TOSS_COIN: bool = true;
// Set, with monsters for the opponent's Raigeki and its Mystical Space Typhoon for this card, and a Trap in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (26202165, MZONE),
    (26202165, MZONE),
    (44095762, GRAVE),
    (12580477, HAND | OPPONENT),
    (5318639, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_COIN + CATEGORY_LEAVE_GRAVE + CATEGORY_SPECIAL_SUMMON + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_DESTROYED);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, effcon);
    e2.set_target(duel, efftg);
    e2.set_operation(duel, effop);
    c.register_effect(duel, e2);
}

fn confilter(c: CardId, duel: &Duel, tp: u8, opp: u8) -> bool {
    c.is_previous_controler(duel, tp)
        && c.is_previous_location(duel, LOCATION_ONFIELD)
        && c.is_previous_position(duel, POS_FACEUP)
        && c.is_reason(duel, REASON_EFFECT)
        && c.is_reason_player(duel, opp)
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.unwrap_or_default()
        .is_exists(duel, |c, duel| confilter(c, duel, tp, 1 - tp), 1, None)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_trap(duel) && c.is_ssetable(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok((duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None))
            || duel.is_existing_matching_card(setfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_COIN, None, 0, tp, 1);
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, None, 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_GRAVE as i32);
    duel.set_possible_operation_info(0, CATEGORY_SET, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let choice = if duel.call_coin(a.tp)? { 1 } else { 2 };
    apply(duel, a, choice)
}

fn effcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_controler(duel, a.tp) && c.is_reason_player(duel, 1 - a.tp) && c.is_reason(duel, REASON_EFFECT)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let option_1 = duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None);
    let option_2 = duel.is_existing_matching_card(setfilter, tp, LOCATION_GRAVE, 0, 1, None);
    if chk == 0 {
        return Ok(option_1 || option_2);
    }
    let choice = duel.select_effect(tp, &[(option_1, aux::stringid(ID, 2)), (option_2, aux::stringid(ID, 3))])?;
    e.set_chain_data(duel, CHOICE, choice.map_or(0, |choice| choice as i64));
    if choice == Some(1) {
        e.set_category(duel, CATEGORY_SPECIAL_SUMMON);
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_GRAVE as i32);
    } else if choice == Some(2) {
        e.set_category(duel, CATEGORY_SET);
        duel.set_operation_info(0, CATEGORY_SET, None, 1, tp, LOCATION_GRAVE as i32);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let choice = a.e.get_chain_data(duel, CHOICE).unwrap_or(0);
    apply(duel, a, choice)
}

fn apply(duel: &mut Duel, a: &Args, choice: i64) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if choice == 1 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    } else if choice == 2 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
        let g = duel.select_matching_card(tp, setfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.sset(tp, g)?;
        }
    }
    Ok(())
}
