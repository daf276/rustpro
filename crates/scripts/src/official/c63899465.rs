// Rescue-ACE HQ
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 63899465;

// In the hand to activate and in a random zone for the others, with Rescue-ACE Impulse and Fire Attacker in the Graveyard to
// shuffle, on the field to power up and a monster of the opponent's to enable the boost.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, 0),
    (38339996, GRAVE),
    (38339996, GRAVE),
    (38339996, GRAVE),
    (64612053, GRAVE),
    (64612053, GRAVE),
    (38339996, MZONE),
    (27125110, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_condition(duel, |duel, a| {
        duel.get_field_group_count(a.e.get_handler_player(duel), 0, LOCATION_MZONE) > 0
    });
    e2.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_RESCUE_ACE]));
    e2.set_value(duel, 500);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 0));
    e4.set_type(duel, EFFECT_TYPE_FIELD);
    e4.set_code(duel, EFFECT_EXTRA_SUMMON_COUNT);
    e4.set_range(duel, LOCATION_FZONE);
    e4.set_target_range(duel, LOCATION_HAND | LOCATION_MZONE, 0);
    e4.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_RESCUE_ACE]));
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 1));
    e5.set_category(duel, CATEGORY_TODECK + CATEGORY_DRAW);
    e5.set_type(duel, EFFECT_TYPE_IGNITION);
    e5.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e5.set_range(duel, LOCATION_FZONE);
    e5.set_count_limit(duel, 1, 0);
    e5.set_target(duel, tdtg);
    e5.set_operation(duel, tdop);
    c.register_effect(duel, e5);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RESCUE_ACE]) && c.is_able_to_deck(duel) && c.is_faceup(duel)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 1) && duel.is_existing_target(cfilter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 4, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, cfilter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 4, 4, None)?;
    duel.set_operation_info(0, CATEGORY_TODECK, g, g.map_or(0, |g| g.len() as u8), tp, 0);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if tg.is_empty() || duel.send_to_deck(tg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let g = duel.get_operated_group();
    if !g.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_DECK | LOCATION_EXTRA), 1, None) {
        return Ok(());
    }
    if g.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_DECK), 1, None) {
        duel.shuffle_deck(tp)?;
    }
    duel.break_effect()?;
    duel.draw(tp, 1, REASON_EFFECT)?;
    Ok(())
}
