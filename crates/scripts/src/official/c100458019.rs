// Raise Moon Blade Shayna
// Raise Moon Lamp Croucier shares the functions generic over the card's id.
use super::c100458016::{draw, draw_target, drcost, register_draw_summon, track_searches};
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 100458019;

// Copies to draw and to discard, and the opponent's Sangan to place on the bottom of the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, DECK),
    (ID, DECK),
    (ID, DECK),
    (ID, DECK),
    (ID, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    register_draw_summon::<1>(duel, c, ID);
    register_summoned_this_way(duel, c, ID, tdtg);
    register_discard_draw::<ID>(duel, c);
    track_searches::<ID, { RESET_PHASE | PHASE_END }>(duel, c);
}

// "If Summoned this way: You can target 1 card your opponent controls; place it on the bottom of the Deck".
pub(crate) fn register_summoned_this_way(duel: &mut Duel, c: CardId, id: u32, tdtg: engine::TargetFn) {
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(id, 1));
    e2.set_category(duel, CATEGORY_TODECK);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_summon_type(duel, &[SUMMON_TYPE_SPECIAL + 1])
    });
    e2.set_target(duel, tdtg);
    e2.set_operation(duel, tdop);
    c.register_effect(duel, e2);
}

// "You can discard this card; draw 1 card", once per turn.
pub(crate) fn register_discard_draw<const ID: u32>(duel: &mut Duel, c: CardId) {
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_DRAW);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_HAND);
    e3.set_count_limit(duel, 1, ID);
    e3.set_cost(duel, |duel, a, chk| {
        Cost::and(duel, a, chk, &[Cost::self_discard, drcost::<ID, 3, { RESET_PHASE | PHASE_END }>])
    });
    e3.set_target(duel, draw_target);
    e3.set_operation(duel, draw);
    c.register_effect(duel, e3);
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| c.is_able_to_deck(duel), tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, |c, duel| c.is_able_to_deck(duel), tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TODECK, g, 1, tp, SEQ_DECKBOTTOM);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.send_to_deck(tc, PLAYER_NONE, SEQ_DECKBOTTOM, REASON_EFFECT)?;
    }
    Ok(())
}
