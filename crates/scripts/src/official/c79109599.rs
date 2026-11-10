// King of the Swamp
use crate::prelude::*;
use crate::testing::{DECK, HAND};

const ID: u32 = 79109599;

pub const LISTED_NAMES: &[u32] = &[CARD_POLYMERIZATION];

pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (CARD_POLYMERIZATION, DECK), (CARD_POLYMERIZATION, DECK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_cost(duel, Cost::self_discard_to_grave);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_FUSION_SUBSTITUTE);
    e2.set_condition(duel, subcon);
    c.register_effect(duel, e2);
}

fn subcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel)
        .is_location(duel, LOCATION_HAND | LOCATION_ONFIELD | LOCATION_GRAVE)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_POLYMERIZATION) && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, a.tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
