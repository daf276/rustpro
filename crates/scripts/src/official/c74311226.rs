// Atlantean Dragoons
use crate::prelude::*;
use crate::testing::{DECK, HAND};

const ID: u32 = 74311226;

pub const LISTED_NAMES: &[u32] = &[ID];
// In the Deck, to be sent to the Graveyard as the cost of Poseidra, the Storming Atlantean in the hand (with Atlantean Pikeman,
// a WATER Normal Monster, to pay its other cost and to search).
pub const SUPPORT: &[(u32, u16)] = &[(ID, DECK), (ID, DECK), (99193444, HAND), (26976414, HAND), (26976414, DECK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_DIRECT_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_level_below(duel, 3) && c.is_race(duel, RACE_SEASERPENT));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_condition(duel, thcon);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

// The condition shared by the Atlanteans that trigger when sent to the Graveyard as the cost of a WATER monster's effect.
pub fn thcon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    a.e.get_handler(duel).is_reason(duel, REASON_COST)
        && re.is_activated(duel)
        && re.is_monster_effect(duel)
        && re.get_handler(duel).is_attribute(duel, ATTRIBUTE_WATER)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    !c.is_code(duel, ID) && c.is_race(duel, RACE_SEASERPENT) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
