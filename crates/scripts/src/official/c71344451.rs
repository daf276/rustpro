// Slash Draw
use crate::prelude::*;
use crate::testing::{DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 71344451;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the hand and the Deck to draw itself, with cards in the Graveyard to return and Sangan on the opponent's field to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, DECK),
    (ID, DECK),
    (ID, DECK),
    (26202165, HAND),
    (26202165, GRAVE),
    (26202165, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(
        duel,
        CATEGORY_DECKDES + CATEGORY_DRAW + CATEGORY_DESTROY + CATEGORY_DAMAGE + CATEGORY_TODECK,
    );
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, Some(c.into())));
    }
    duel.discard_hand(tp, CardId::is_discardable, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let ct = duel.get_field_group_count(tp, 0, LOCATION_ONFIELD);
    if chk == 0 {
        return Ok(ct > 0
            && duel.get_field_group_count(tp, LOCATION_DECK, 0) > ct
            && duel.is_player_can_discard_deck(tp, ct)
            && duel.is_player_can_draw(tp, 1));
    }
    duel.set_operation_info(0, CATEGORY_DECKDES, None, 0, tp, ct as i32);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, None, 1, PLAYER_ALL, LOCATION_ONFIELD as i32);
    duel.set_possible_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 2000);
    duel.set_possible_operation_info(0, CATEGORY_TODECK, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let ct = duel.get_field_group_count(tp, 0, LOCATION_ONFIELD);
    if ct == 0 || duel.discard_deck(tp, ct as u16, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let grave_ct = duel
        .get_operated_group()
        .filter_count(duel, |c, duel| c.is_location(duel, LOCATION_GRAVE), None);
    if grave_ct == 0 || !duel.is_player_can_draw(tp, 0) {
        return Ok(());
    }
    duel.break_effect()?;
    if duel.draw(tp, 1, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let Some(tc) = duel.get_operated_group().get_first() else {
        return Ok(());
    };
    duel.confirm_cards(1 - tp, tc)?;
    if tc.is_code(duel, ID) && duel.send_to_grave(tc, REASON_EFFECT)? > 0 && tc.is_location(duel, LOCATION_GRAVE) {
        let sg = duel.get_matching_group(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, Some(e.get_handler(duel).into()));
        if sg.is_empty() || duel.destroy(sg, REASON_EFFECT)? == 0 {
            return Ok(());
        }
        let tg = duel
            .get_operated_group()
            .filter(duel, |c, duel| c.is_location(duel, LOCATION_GRAVE), None);
        if !tg.is_empty() {
            duel.break_effect()?;
            duel.damage(1 - tp, tg.len() as u32 * 2000, REASON_EFFECT)?;
        }
    } else {
        duel.shuffle_hand(tp)?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
        let filter = aux::necro_valley_filter(CardId::is_able_to_deck);
        let dg = duel.select_matching_card(tp, filter, tp, LOCATION_GRAVE, 0, grave_ct as u8, grave_ct as u8, None)?;
        if !dg.is_empty() {
            duel.hint_selection_full(dg, true);
            duel.send_to_deck(dg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
        }
    }
    Ok(())
}
