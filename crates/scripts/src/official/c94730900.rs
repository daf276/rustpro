// Infernoble Knight Maugis
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 94730900;

pub const LISTED_SERIES: &[u16] = &[SET_NOBLE_ARMS];

// In the Deck for Infernoble Knight Ogier to send, with Elemental HERO Burstinatrix (a FIRE Warrior) in the GY to shuffle and on
// the field to equip.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, DECK),
    (ID, DECK),
    (ID, GRAVE),
    (21351206, HAND),
    (21351206, HAND),
    (58932615, GRAVE),
    (58932615, GRAVE),
    (58932615, GRAVE),
    (37478723, GRAVE),
    (58932615, MZONE),
    (58932615, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, drcost);
    e1.set_target(duel, stock::draw_target::<1>);
    e1.set_operation(duel, stock::draw);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_category(duel, CATEGORY_EQUIP);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_target(duel, super::c21351206::eqtg);
    e2.set_operation(duel, super::c21351206::eqop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_EQUIP);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e3.set_value(duel, 1);
    c.register_effect(duel, e3);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    ((c.is_race(duel, RACE_WARRIOR) && c.is_attribute(duel, ATTRIBUTE_FIRE)) || c.is_set_card(duel, &[SET_NOBLE_ARMS]))
        && (c.is_location(duel, LOCATION_GRAVE) || c.is_faceup(duel))
        && c.is_able_to_deck_as_cost(duel)
}

fn drcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    let location = LOCATION_GRAVE | LOCATION_REMOVED;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(cfilter, tp, location, 0, 3, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_matching_card(tp, cfilter, tp, location, 0, 3, 3, Some(c.into()))?;
    duel.send_to_deck(g, PLAYER_NONE, 3, REASON_COST)?;
    Ok(false)
}
