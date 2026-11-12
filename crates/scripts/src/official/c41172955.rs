// Green Gadget
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 41172955;
const RED_GADGET: u32 = 86445415;

pub const LISTED_NAMES: &[u32] = &[RED_GADGET];

// Four copies in the hand to Normal Summon, Red Gadget in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (RED_GADGET, DECK),
    (RED_GADGET, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e1.set_type(duel, EFFECT_TYPE_TRIGGER_O + EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_target(duel, tg::<RED_GADGET>);
    e1.set_operation(duel, op::<RED_GADGET>);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
}

fn filter<const CODE: u32>(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CODE) && c.is_able_to_hand(duel)
}

pub fn tg<const CODE: u32>(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, filter::<CODE>, LOCATION_DECK, CATEGORY_TOHAND)
}

pub fn op<const CODE: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if let Some(tc) = duel.get_first_matching_card(filter::<CODE>, tp, LOCATION_DECK, 0, None) {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, tc)?;
    }
    Ok(())
}
