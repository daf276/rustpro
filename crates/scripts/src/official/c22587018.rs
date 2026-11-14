// Hydrogeddon
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 22587018;

pub const LISTED_NAMES: &[u32] = &[ID];

// Copies in the hand to summon and in the Deck to Special Summon, and the opponent's Volcanic Rats (500 ATK and DEF) to destroy
// by battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (ID, DECK),
    (ID, DECK),
    (33112041, MZONE | OPPONENT),
    (33112041, MZONE | OPPONENT),
    (33112041, MZONE | OPPONENT),
    (33112041, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_code(duel, EVENT_BATTLE_DESTROYING);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_condition(duel, aux::bdogcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, ID) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    stock::special_summon_target(duel, a, chk, |c, duel| filter(c, duel, e, tp), LOCATION_DECK)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    if let Some(tc) = duel.get_first_matching_card(|c, duel| filter(c, duel, e, tp), tp, LOCATION_DECK, 0, None) {
        duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
