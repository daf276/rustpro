// Clown Crew Flair
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 42759961;

// In the hand and on the field, with Clown Crew Biancaviso in the hand to Tribute Summon with it and one in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, MZONE), (82159583, HAND), (82159583, DECK)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_HANDES);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::self_reveal);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_RELEASE);
    e2.set_target(duel, efftg);
    e2.set_operation(duel, effop);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_CLOWN_CREW]) && !c.is_ritual_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, tp, 1);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first()
        && duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? > 0
        && tc.is_location(duel, LOCATION_HAND)
    {
        duel.confirm_cards(1 - tp, g)?;
        duel.shuffle_hand(tp)?;
        duel.break_effect()?;
        duel.discard_hand(tp, aux::TRUE, 1, 1, REASON_EFFECT | REASON_DISCARD)?;
    }
    Ok(())
}

fn ritualfilter(c: CardId, duel: &Duel) -> bool {
    c.is_ritual_monster(duel) && c.is_faceup(duel) && c.is_able_to_deck(duel)
}

fn turnsetfilter(c: CardId, duel: &Duel) -> bool {
    c.is_effect_monster(duel) && c.is_can_turn_set(duel)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let b1 = !duel.has_flag_effect(tp, ID)
        && duel.is_existing_matching_card(
            ritualfilter,
            tp,
            LOCATION_MZONE | LOCATION_GRAVE,
            LOCATION_MZONE | LOCATION_GRAVE,
            1,
            None,
        );
    let b2 =
        !duel.has_flag_effect(tp, ID + 1) && duel.is_existing_matching_card(turnsetfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
        e.set_category(duel, CATEGORY_TODECK);
        duel.set_operation_info(0, CATEGORY_TODECK, None, 1, PLAYER_ALL, (LOCATION_MZONE | LOCATION_GRAVE) as i32);
    } else if op == Some(2) {
        duel.register_flag_effect(tp, ID + 1, RESET_PHASE | PHASE_END, 0, 1);
        e.set_category(duel, CATEGORY_POSITION + CATEGORY_SET);
        duel.set_operation_info(0, CATEGORY_POSITION, None, 1, PLAYER_EITHER, POS_FACEDOWN_DEFENSE as i32);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    match a.e.get_label(duel) {
        1 => {
            let g = duel.get_matching_group(
                ritualfilter,
                tp,
                LOCATION_MZONE | LOCATION_GRAVE,
                LOCATION_MZONE | LOCATION_GRAVE,
                None,
            );
            if !g.is_empty() {
                duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
            }
        }
        2 => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
            let g = duel.select_matching_card(tp, turnsetfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
            if !g.is_empty() {
                duel.hint_selection(g);
                duel.change_position(g, POS_FACEDOWN_DEFENSE)?;
            }
        }
        _ => {}
    }
    Ok(())
}
