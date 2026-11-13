// Witchcrafter Walpurgis
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 32353566;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the hand to activate, with Witchcrafter Haine and Pot of Greed to discard on the field, and Witchcrafter monsters and Spells in the
// Deck to add or send instead.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (84523092, MZONE),
    (55144522, HAND),
    (55144522, HAND),
    (64756282, DECK),
    (21744288, DECK),
    (57916305, DECK),
    (57916305, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_HANDES);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_FZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = Witchcrafter::create_cost_replace_effect(duel, c);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_condition(duel, repcon);
    e2.set_cost_replace_operation(duel, repop);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_WITCHCRAFTER]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_HANDES, None, 1, tp, 1);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty()
        && duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? > 0
        && g.get_first().is_some_and(|tc| tc.is_location(duel, LOCATION_HAND))
    {
        duel.confirm_cards(1 - tp, g)?;
        duel.shuffle_hand(tp)?;
        duel.break_effect()?;
        duel.discard_hand(tp, aux::TRUE, 1, 1, REASON_EFFECT | REASON_DISCARD)?;
    }
    Ok(())
}

fn repcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_WITCHCRAFTER]) && c.is_spell_trap(duel) && !c.is_code(duel, ID) && c.is_able_to_grave_as_cost(duel)
}

fn repcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.e.get_handler_player(duel);
    duel.is_turn_player(tp) && duel.is_existing_matching_card(repcostfilter, tp, LOCATION_DECK, 0, 1, None)
}

fn repop(duel: &mut Duel, _: EffectId, call: CostReplaceCall) -> Script<()> {
    let tp = call.args.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, repcostfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_grave(g, REASON_COST)?;
    }
    Ok(())
}
