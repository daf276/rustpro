// The Hidden Hecahands
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 20415050;
const PAID_COST: u8 = 0;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the hand and the Graveyard, with a card on the field to send, Hecahands Gaigas to add and Yad'al-Hecahands to Set.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (95132593, MZONE),
    (95132593, DECK),
    (95132593, DECK),
    (29792472, DECK),
    (29792472, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, thcost);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DRAW + CATEGORY_HANDES);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, drwtg);
    e2.set_operation(duel, drwop);
    c.register_effect(duel, e2);
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if duel.is_existing_matching_card(CardId::is_able_to_grave_as_cost, tp, LOCATION_ONFIELD, 0, 1, Some(c.into()))
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
        let g = duel.select_matching_card(tp, CardId::is_able_to_grave_as_cost, tp, LOCATION_ONFIELD, 0, 1, 1, Some(c.into()))?;
        duel.send_to_grave(g, REASON_COST)?;
        a.e.set_chain_data(duel, PAID_COST, 1);
    }
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_HECAHANDS]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_HECAHANDS]) && c.is_spell_trap(duel) && c.is_ssetable(duel) && !c.is_code(duel, ID)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() && duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? > 0 {
        duel.confirm_cards(1 - tp, g)?;
        duel.shuffle_hand(tp)?;
        if a.e.get_chain_data(duel, PAID_COST).is_some()
            && duel.is_existing_matching_card(setfilter, tp, LOCATION_DECK, 0, 1, None)
            && duel.select_yes_no(tp, aux::stringid(ID, 3))?
        {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
            let sg = duel.select_matching_card(tp, setfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
            if !sg.is_empty() {
                duel.break_effect()?;
                duel.sset(tp, sg)?;
            }
        }
    }
    Ok(())
}

fn drwtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_draw(1 - tp, 1));
    }
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, 1 - tp, 1);
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, 1 - tp, 1);
    Ok(false)
}

fn drwop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.draw(1 - tp, 1, REASON_EFFECT)? > 0 {
        duel.shuffle_hand(1 - tp)?;
        duel.break_effect()?;
        duel.discard_hand(1 - tp, aux::TRUE, 1, 1, REASON_EFFECT | REASON_DISCARD)?;
    }
    Ok(())
}
