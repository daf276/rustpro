// Flying Kamakiri #1
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE, OPPONENT};

const ID: u32 = 84834865;

// Copies on the field to be destroyed in battle by the opponent's Blue-Eyes White Dragons, with Flying Kamakiri #2 (a
// Normal Monster) in the Deck to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (CARD_BLUEEYES_W_DRAGON, MZONE | OPPONENT),
    (CARD_BLUEEYES_W_DRAGON, MZONE | OPPONENT),
    (3134241, DECK),
    (3134241, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BATTLE_DESTROYED);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_location(duel, LOCATION_GRAVE));
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_attack_below(duel, 1500) && c.is_attribute(duel, ATTRIBUTE_WIND) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| filter(c, duel, a.e, a.tp), LOCATION_DECK)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| filter(c, duel, a.e, a.tp), LOCATION_DECK, POS_FACEUP_ATTACK)
}
