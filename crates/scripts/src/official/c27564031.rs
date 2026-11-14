// Malefic World
use crate::keys::{REPLACE_COUNT, REPLACE_MAX};
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 27564031;

pub const LISTED_SERIES: &[u16] = &[SET_MALEFIC];
// Three Malefic monsters in the Deck for the field to search, which draws a card at the start of the turn.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (48829461, DECK), (48829461, DECK), (74509280, DECK), (74509280, DECK)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_PREDRAW);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_condition(duel, condition);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    a.tp == duel.get_turn_player() && duel.get_field_group_count(a.tp, LOCATION_DECK, 0) > 0 && duel.get_draw_count(a.tp) > 0
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MALEFIC]) && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_DECK, 0, 3, None));
    }
    let dt = duel.get_draw_count(tp);
    if dt != 0 {
        duel.set_script_state(REPLACE_COUNT, 0);
        duel.set_script_state(REPLACE_MAX, i64::from(dt));
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
        e1.set_code(duel, EFFECT_DRAW_COUNT);
        e1.set_target_range(duel, 1, 0);
        e1.set_reset(duel, RESET_PHASE | PHASE_DRAW);
        e1.set_value(duel, 0);
        duel.register_effect(e1, tp);
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, 0, LOCATION_DECK as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let count = duel.get_script_state(REPLACE_COUNT) + 1;
    duel.set_script_state(REPLACE_COUNT, count);
    if count > duel.get_script_state(REPLACE_MAX) || !e.get_handler(duel).is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let g = duel.get_matching_group(filter, tp, LOCATION_DECK, 0, None);
    if g.len() >= 3 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let sg = g.select(duel, tp, 3, 3, None)?;
        duel.confirm_cards(1 - tp, sg)?;
        duel.shuffle_deck(tp)?;
        let tg = sg.random_select(duel, 1 - tp, 1);
        duel.send_to_hand(tg, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
