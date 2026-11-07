// Burst Return
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 27191436;
pub const LISTED_NAMES: &[u32] = &[58932615];

// In the hand with Elemental HERO Burstinatrix on the field and Elemental HERO Avian to return.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (58932615, MZONE), (21844576, MZONE)];
pub const CHECKS: &[Check] = &[Check::Resolves];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_code(duel, 58932615)),
            a.tp,
            LOCATION_ONFIELD,
            0,
            1,
            None,
        )
    });
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_ELEMENTAL_HERO]) && !c.is_code(duel, 58932615) && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    let g = duel.get_matching_group(filter, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(filter, a.tp, LOCATION_MZONE, LOCATION_MZONE, None);
    duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
    Ok(())
}
