// Nimble Beaver
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND};

const ID: u32 = 68353324;

pub const LISTED_SERIES: &[u16] = &[SET_NIMBLE];
// Copies in the hand to Normal Summon, Nimble Momonga in the Deck and the Graveyard to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (22567609, DECK),
    (22567609, DECK),
    (22567609, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(22567609)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_NIMBLE]) && c.is_level_below(duel, 3) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| filter(c, duel, a.e, a.tp), LOCATION_DECK | LOCATION_GRAVE)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    stock::special_summon(
        duel,
        a,
        aux::necro_valley_filter(|c, duel| filter(c, duel, e, tp)),
        LOCATION_DECK | LOCATION_GRAVE,
        POS_FACEUP,
    )
}
