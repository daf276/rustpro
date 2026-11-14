// Raiden, Hand of the Lightsworn
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE};

const ID: u32 = 77558536;

pub const LISTED_SERIES: &[u16] = &[SET_LIGHTSWORN];
// On the field with Lightsworn monsters (Lyla, Wulf, Jain) in the Deck to send to the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (22624373, DECK), (22624373, DECK), (58996430, DECK), (96235275, DECK)];

pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DECKDES);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DECKDES);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_PHASE + PHASE_END);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, |duel, a| a.tp == duel.get_turn_player());
    e2.set_target(duel, distg);
    e2.set_operation(duel, |duel, a| {
        duel.discard_deck(a.tp, 2, REASON_EFFECT)?;
        Ok(())
    });
    c.register_effect(duel, e2);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_player_can_discard_deck(a.tp, 2));
    }
    duel.set_operation_info(0, CATEGORY_DECKDES, None, 0, a.tp, 2);
    Ok(false)
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_location(duel, LOCATION_GRAVE) && c.is_set_card(duel, &[SET_LIGHTSWORN]) && c.is_monster(duel)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    duel.discard_deck(tp, 2, REASON_EFFECT)?;
    let g = duel.get_operated_group();
    if g.filter_count(duel, cfilter, None) == 0 {
        return Ok(());
    }
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, a.e) {
        duel.break_effect()?;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, 200);
        e1.set_reset_full(duel, RESETS_STANDARD_DISABLE_PHASE_END, 2);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DECKDES, None, 0, a.tp, 2);
    Ok(false)
}
