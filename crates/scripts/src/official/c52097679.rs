// Shield & Sword
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 52097679;

// Sangan on both sides to swap.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (26202165, MZONE), (26202165, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DEFCHANGE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(a.e.is_has_type(duel, EFFECT_TYPE_ACTIVATE)
            && duel.is_existing_matching_card(|c, duel| c.is_faceup(duel), tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    let g = duel.get_matching_group(|c, duel| c.is_faceup(duel), tp, LOCATION_MZONE, LOCATION_MZONE, None);
    duel.set_target_card(g);
    Ok(false)
}

fn filter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_faceup(duel) && c.is_relate_to_effect(duel, e) && !c.is_immune_to_effect(duel, e)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let sg = duel.get_matching_group(|c, duel| filter(c, duel, e), tp, LOCATION_MZONE, LOCATION_MZONE, None);
    let c = e.get_handler(duel);
    for tc in sg {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SWAP_BASE_AD);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
