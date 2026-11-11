// Emissary of the Afterlife
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 75043725;

// On the field to be destroyed by the opponent's Raigeki, with Normal Monsters of Level 3 or lower (Change Slime) in both Decks.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (12580477, HAND | OPPONENT),
    (18914778, DECK),
    (18914778, DECK),
    (18914778, DECK | OPPONENT),
    (18914778, DECK | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_previous_location(duel, LOCATION_ONFIELD));
    e1.set_target(duel, stock::operation_info_target::<CATEGORY_TOHAND, LOCATION_DECK>);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_level_below(duel, 3) && c.is_type(duel, TYPE_NORMAL) && c.is_able_to_hand(duel)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let mut g1 = duel.select_matching_card(tp, filter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    let tc1 = g1.get_first();
    duel.hint(HINT_SELECTMSG, 1 - tp, HINTMSG_ATOHAND);
    let g2 = duel.select_matching_card(1 - tp, filter, tp, 0, LOCATION_DECK, 1, 1, None)?;
    let tc2 = g2.get_first();
    g1.merge(g2);
    duel.send_to_hand(g1, PLAYER_NONE, REASON_EFFECT)?;
    if let Some(tc1) = tc1 {
        duel.confirm_cards(1 - tp, tc1)?;
    }
    if let Some(tc2) = tc2 {
        duel.confirm_cards(tp, tc2)?;
    }
    Ok(())
}
