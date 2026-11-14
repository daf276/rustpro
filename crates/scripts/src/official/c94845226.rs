// The Fabled Rubyruda
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 94845226;

pub const LISTED_SERIES: &[u16] = &[SET_FABLED];
// Two on the field with Fabled Lurrie in the hand to discard, and attackers of the opponent's.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (97651498, HAND),
    (97651498, HAND),
    (CARD_CYBER_DRAGON, MZONE | OPPONENT),
    (CARD_CYBER_DRAGON, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BE_BATTLE_TARGET);
    e1.set_cost(duel, cost);
    e1.set_operation(duel, |duel, _| {
        duel.negate_attack()?;
        Ok(())
    });
    c.register_effect(duel, e1);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_FABLED]) && c.is_monster(duel) && c.is_discardable(duel)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(cfilter, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, cfilter, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}
