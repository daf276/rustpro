// Megalith Hagith
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

use super::c36849933::Params;

const ID: u32 = 90444325;

pub const LISTED_SERIES: &[u16] = &[SET_MEGALITH];

// On the field to Ritual Summon Megalith Och or a copy from the hand with itself, Och there to summon it the same way, and Megalith
// Unformed and Megalith Anastasis in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, HAND),
    (36849933, MZONE),
    (36849933, HAND),
    (69003792, DECK),
    (29876299, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_ritual_summoned(duel));
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_RELEASE + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, Ritual::target::<Params, RITPROC_GREATER>);
    e2.set_operation(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        if c.is_relate_to_effect(duel, a.e) && c.is_controler(duel, a.tp) {
            Ritual::operation::<Params, RITPROC_GREATER>(duel, a)?;
        }
        Ok(())
    });
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MEGALITH]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
