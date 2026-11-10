// Tempest, Dragon Ruler of Storms
use super::c26400609::{add_banished_search_effect, add_summon_and_return_effects};
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND};

const ID: u32 = 89399912;

// Copies in the hand, the Graveyard and the Deck (to be banished as a cost, then searched), with Harpie Lady (WIND)
// and Red-Eyes Black Dragon (Dragon) as the cost cards and the Deck monster to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (ID, DECK),
    (ID, DECK),
    (76812113, HAND),
    (74677422, HAND),
    (74677422, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
    Check::ResolvesEffect(aux::stringid(ID, 3)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    add_summon_and_return_effects::<ID, ATTRIBUTE_WIND>(duel, c);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_HAND);
    e3.set_count_limit(duel, 1, ID);
    e3.set_cost(duel, DragonRuler::self_discard_to_grave_cost::<ATTRIBUTE_WIND>);
    e3.set_target(duel, dthtg);
    e3.set_operation(duel, dthop);
    c.register_effect(duel, e3);
    add_banished_search_effect::<ID, ATTRIBUTE_WIND>(duel, c);
}

fn dthfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_DRAGON) && c.is_able_to_hand(duel)
}

fn dthtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(dthfilter, tp, LOCATION_DECK, 0, 1, Some(a.e.get_handler(duel).into())));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn dthop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, dthfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
