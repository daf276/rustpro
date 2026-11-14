// Gamma The Electromagnet Warrior
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 15502037;

pub const LISTED_SERIES: &[u16] = &[SET_MAGNET_WARRIOR];
pub const LISTED_NAMES: &[u32] = &[ID];

// Alpha The Magnet Warrior (Level 4) in the hand and Deck to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (99785935, HAND),
    (99785935, HAND),
    (99785935, DECK),
    (99785935, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_TRIGGER_O + EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    super::c42023223::register_tribute_summon::<ID>(duel, c);
}

fn filter(e: EffectId, tp: u8) -> impl Fn(CardId, &Duel) -> bool {
    move |c, duel| {
        c.is_set_card(duel, &[SET_MAGNET_WARRIOR])
            && !c.is_code(duel, ID)
            && c.is_level_below(duel, 4)
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
    }
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, filter(a.e, a.tp), LOCATION_HAND)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, filter(a.e, a.tp), LOCATION_HAND, POS_FACEUP)
}
