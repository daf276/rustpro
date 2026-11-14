// Chain Resonator
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 13764881;

pub const LISTED_SERIES: &[u16] = &[SET_RESONATOR];
pub const LISTED_NAMES: &[u32] = &[ID];

// In the hand beside a Synchro Monster (Red Dragon Archfiend), with Soul Resonators in the Deck to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (CARD_RED_DRAGON_ARCHFIEND, MZONE),
    (62991792, DECK),
    (62991792, DECK),
    (62991792, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(
        aux::faceup_filter(|c, duel| c.is_type(duel, TYPE_SYNCHRO)),
        a.tp,
        LOCATION_MZONE,
        LOCATION_MZONE,
        1,
        None,
    )
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_RESONATOR]) && !c.is_code(duel, ID) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| filter(c, duel, a.e, a.tp), LOCATION_DECK)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| filter(c, duel, a.e, a.tp), LOCATION_DECK, POS_FACEUP)
}
