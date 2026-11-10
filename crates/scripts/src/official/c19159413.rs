// De-Spell
use crate::prelude::*;
use crate::testing::{Check, HAND, SZONE};

const ID: u32 = 19159413;

// Face-down Bottomless Trap Holes and Raigeki set in the Spell/Trap Zones to target.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (29401950, SZONE), (29401950, SZONE), (12580477, SZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_facedown(duel) || c.is_spell(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_SZONE) && filter(chkc, duel) && chkc != c);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_SZONE, LOCATION_SZONE, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, filter, tp, LOCATION_SZONE, LOCATION_SZONE, 1, 1, Some(c.into()))?;
    if g.and_then(|g| g.get_first()).is_some_and(|tc| tc.is_faceup(duel)) {
        duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    }
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (tp, e) = (a.tp, a.e);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
    {
        if tc.is_facedown(duel) {
            duel.confirm_cards(tp, tc)?;
        }
        if tc.is_spell(duel) {
            duel.destroy(tc, REASON_EFFECT)?;
        }
    }
    Ok(())
}
