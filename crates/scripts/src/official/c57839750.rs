// Mother Grizzly
use crate::prelude::*;
use crate::testing::{DECK, MZONE, OPPONENT};

const ID: u32 = 57839750;

// On the field to be destroyed in battle by strong opponent monsters, with WATER monsters of 1500 ATK or less (Jellyfish,
// Island Turtle, Flying Penguin) in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (48766543, MZONE | OPPONENT),
    (70781052, MZONE | OPPONENT),
    (67371383, MZONE | OPPONENT),
    (6631034, MZONE | OPPONENT),
    (14851496, DECK),
    (14851496, DECK),
    (4042268, DECK),
    (5628232, DECK),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BATTLE_DESTROYED);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_location(duel, LOCATION_GRAVE)
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_attack_below(duel, 1500) && c.is_attribute(duel, ATTRIBUTE_WATER) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| filter(c, duel, a.e, a.tp), LOCATION_DECK)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| filter(c, duel, a.e, a.tp), LOCATION_DECK, POS_FACEUP_ATTACK)
}
