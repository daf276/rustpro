// Babycerasaurus
use crate::prelude::*;
use crate::testing::{DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 36042004;

// On the field against the opponent's Raigeki, with Dinosaurs (Uraby, Trakodon) in the Deck to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (12580477, HAND | OPPONENT),
    (1784619, DECK),
    (1784619, DECK),
    (42348802, DECK),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e1.set_code(duel, EVENT_TO_GRAVE);
    e1.set_condition(duel, |_, a| {
        a.r & (REASON_DESTROY | REASON_EFFECT) == REASON_DESTROY | REASON_EFFECT
    });
    e1.set_target(duel, stock::operation_info_target::<CATEGORY_SPECIAL_SUMMON, LOCATION_DECK>);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_below(duel, 4) && c.is_race(duel, RACE_DINOSAUR) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| filter(c, duel, a.e, a.tp), LOCATION_DECK, POS_FACEUP)
}
