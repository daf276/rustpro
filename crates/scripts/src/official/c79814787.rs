// The White Stone of Legend
use crate::prelude::*;
use crate::testing::{DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 79814787;

pub const LISTED_NAMES: &[u32] = &[CARD_BLUEEYES_W_DRAGON];

// Blue-Eyes White Dragon in the Deck, and the opponent's Dark Hole to send it to the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, HAND),
    (CARD_BLUEEYES_W_DRAGON, DECK),
    (CARD_BLUEEYES_W_DRAGON, DECK),
    (53129443, HAND | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_target(duel, stock::operation_info_target::<CATEGORY_TOHAND, LOCATION_DECK>);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_BLUEEYES_W_DRAGON) && c.is_able_to_hand(duel)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    if let Some(tc) = duel.get_first_matching_card(filter, tp, LOCATION_DECK, 0, None) {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, tc)?;
    }
    Ok(())
}
