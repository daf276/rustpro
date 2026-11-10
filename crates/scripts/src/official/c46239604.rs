// Dupe Frog
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 46239604;
const DES_FROG: u32 = 84451804;

pub const LISTED_NAMES: &[u32] = &[ID, DES_FROG];

// On the field, destroyed by the opponent's Dark Hole, with Swap Frog in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (9126351, DECK),
    (9126351, DECK),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_CHANGE_CODE);
    e1.set_value(duel, DES_FROG as i32);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_previous_location(duel, LOCATION_ONFIELD));
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_CANNOT_SELECT_BATTLE_TARGET);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, 0, LOCATION_MZONE);
    e3.set_value(duel, Value::Card(|duel, e, c| i32::from(c != e.get_handler(duel))));
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_FROG]) && c.is_monster(duel) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, a.tp, LOCATION_GRAVE | LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, a.tp, (LOCATION_GRAVE | LOCATION_DECK) as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(thfilter),
        tp,
        LOCATION_DECK | LOCATION_GRAVE,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
