// Cyber Emergency
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, OPPONENT, SZONE};

const ID: u32 = 60600126;

// In the hand with Cyber Dragon to search, and the opponent's set Solemn Judgment to negate it.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (CARD_CYBER_DRAGON, DECK), (41420027, SZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_CHAIN_NEGATED);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_condition(duel, thcond);
    e2.set_cost(duel, thcost);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel)
        && c.is_able_to_hand(duel)
        && (c.is_set_card(duel, &[SET_CYBER_DRAGON])
            || (c.is_attribute(duel, ATTRIBUTE_LIGHT) && c.is_race(duel, RACE_MACHINE) && !c.is_summonable_card(duel)))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, filter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g1 = duel.select_matching_card(tp, filter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g1.is_empty() {
        duel.send_to_hand(g1, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g1)?;
    }
    Ok(())
}

fn thcond(duel: &Duel, a: &Args) -> bool {
    let de: Option<EffectId> = duel.get_chain_info(a.ev as u8, CHAININFO_DISABLE_REASON);
    let dp: u8 = duel.get_chain_info(a.ev as u8, CHAININFO_DISABLE_PLAYER);
    a.re.is_some_and(|re| re.get_handler(duel) == a.e.get_handler(duel) && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE))
        && a.rp == a.tp
        && de.is_some()
        && dp != a.tp
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, Some(c.into())));
    }
    duel.discard_hand(tp, CardId::is_discardable, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, a.tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.send_to_hand(c, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - a.tp, c)?;
    }
    Ok(())
}
