// Cyberdark Claw
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 82562802;

pub const LISTED_SERIES: &[u16] = &[SET_CYBERDARK];
// In the hand with Cyberdark Realms in the Deck to search, and in the Graveyard to be equipped by Cyberdark Horn, with Cyber End Dragon in
// the Extra Deck to send and Cyber Dragons of the opponent's to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (41230939, HAND),
    (1546123, EXTRA),
    (64753988, DECK),
    (64753988, DECK),
    (70095154, MZONE | OPPONENT),
    (70095154, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_condition(duel, thgycon);
    e1.set_target(duel, thgytg);
    e1.set_operation(duel, thgyop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, Cost::self_discard);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_TOGRAVE);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_PRE_DAMAGE_CALCULATE);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, tgcon);
    e3.set_target(duel, tgtg);
    e3.set_operation(duel, tgop);
    c.register_effect(duel, e3);
}

fn thgyfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_CYBERDARK]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thgycon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_location(duel, LOCATION_SZONE) && c.get_previous_equip_target(duel).is_some() && !c.is_reason(duel, REASON_LOST_TARGET)
}

fn thgytg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && thgyfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thgyfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thgyfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, tp, 0);
    Ok(false)
}

fn thgyop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - a.tp, tc)?;
    }
    Ok(())
}

fn thdeckfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_CYBERDARK]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thdeckfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let tg = duel.select_matching_card(tp, thdeckfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !tg.is_empty() {
        duel.send_to_hand(tg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, tg)?;
    }
    Ok(())
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_able_to_grave(duel)
}

fn tgcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel)
        .get_equip_target(duel)
        .is_some_and(|ec| Some(ec) == duel.get_attacker() || Some(ec) == duel.get_attack_target())
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, tgfilter, LOCATION_EXTRA, CATEGORY_TOGRAVE)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::send_to_grave(duel, a, tgfilter, LOCATION_EXTRA)
}
