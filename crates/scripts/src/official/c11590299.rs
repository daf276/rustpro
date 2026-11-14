// Dora Dora
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 11590299;

pub const LISTED_NAMES: &[u32] = &[ID];

// Copies in the hand to Normal Summon, and FIRE Dragons (copies, Flamvell Guard) in the Deck to add and excavate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, DECK),
    (21615956, DECK),
    (21615956, DECK),
    (21615956, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DECKDES + CATEGORY_ATKCHANGE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_target(duel, exctg);
    e2.set_operation(duel, excop);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level_below(duel, 4) && c.is_attribute(duel, ATTRIBUTE_FIRE) && c.is_race(duel, RACE_DRAGON) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn exctg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk != 0 || duel.is_player_can_discard_deck(a.tp, 1))
}

fn excop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !duel.is_player_can_discard_deck(tp, 1) {
        return Ok(());
    }
    duel.confirm_decktop(tp, 1)?;
    let Some(tc) = duel.get_decktop_group(tp, 1).get_first() else {
        return Ok(());
    };
    if tc.is_attribute(duel, ATTRIBUTE_FIRE) && tc.is_race(duel, RACE_DRAGON) {
        duel.disable_shuffle_check(true);
        if duel.send_to_grave(tc, REASON_EFFECT | REASON_EXCAVATE)? > 0 {
            let ct = duel.get_matching_group_count(aux::faceup_filter(|c, duel| c.is_code(duel, ID)), tp, LOCATION_MZONE, 0, None);
            a.e.get_handler(duel).update_attack(duel, ct as i32 * 1000, None, None, None);
        }
    } else {
        duel.move_to_deck_bottom(tc.into())?;
    }
    Ok(())
}
