// Armed Ninja
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE, OPPONENT, SZONE};

const ID: u32 = 9076207;

// Face-down copies flipped by the opponent's attacker (Lucina), with Spells and Traps (Mystical Space Typhoon) on the field to
// destroy, and Dark Hole in the Deck so it is not drawn to hand.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (13597785, MZONE | OPPONENT),
    (5318639, SZONE),
    (5318639, SZONE | OPPONENT),
    (53129443, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_FLIP);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_facedown(duel) || c.is_spell(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_SZONE) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(true);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, filter, tp, LOCATION_SZONE, LOCATION_SZONE, 1, 1, None)?;
    if g.and_then(|g| g.get_first()).is_some_and(|tc| tc.is_faceup(duel)) {
        duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    }
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        if tc.is_facedown(duel) {
            duel.confirm_cards(a.tp, tc)?;
        }
        if tc.is_spell(duel) {
            duel.destroy(tc, REASON_EFFECT)?;
        }
    }
    Ok(())
}
