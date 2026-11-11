// Tri-Brigade Mercourier
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 19096726;

pub const LISTED_NAMES: &[u32] = &[CARD_ALBAZ, ID];

// In the hand, on the field and in the Graveyard with Albion the Branded Dragon to negate the opponent's monster effects,
// Fallen of Albaz in the Deck to search, and Tri-Brigade Fraktall to banish it from the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (ID, GRAVE),
    (87746184, MZONE),
    (68468459, DECK),
    (68468459, DECK),
    (87209160, MZONE),
    (99726621, EXTRA),
    (6297941, GRAVE),
    (6297941, GRAVE),
    (6297941, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_HAND | LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, negcon);
    e1.set_cost(duel, Cost::self_to_grave);
    e1.set_target(duel, negtg);
    e1.set_operation(duel, negop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_REMOVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn negconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_FUSION) && c.lists_code_as_material(duel, &[CARD_ALBAZ])
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    a.rp == 1 - a.tp
        && a.re.is_some_and(|re| re.is_monster_effect(duel))
        && duel.is_existing_matching_card(negconfilter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        let disabled = a.re.is_some_and(|re| re.get_handler(duel).is_status(duel, STATUS_DISABLED));
        return Ok(duel.is_chain_disablable(a.ev as u8) && !disabled);
    }
    duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, 0, 0);
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.negate_effect(a.ev as u8);
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_code(duel, CARD_ALBAZ) || (c.is_monster(duel) && c.lists_code(duel, CARD_ALBAZ)))
        && c.is_able_to_hand(duel)
        && !c.is_code(duel, ID)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
