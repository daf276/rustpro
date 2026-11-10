// Gravedigger Ghoul
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 82542267;

// In the hand, with Sangan in the opponent's monster zone and Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, GRAVE | OPPONENT),
    (26202165, GRAVE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_remove(duel) && c.is_monster(duel) && aux::sp_elim_filter(c, duel, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE | LOCATION_GRAVE) && chkc.is_controler(duel, 1 - tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, 0, LOCATION_MZONE | LOCATION_GRAVE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, filter, tp, 0, LOCATION_MZONE | LOCATION_GRAVE, 1, 2, None)?;
    let count = g.map_or(0, |g| g.len() as u8);
    duel.set_operation_info(0, CATEGORY_REMOVE, g, count, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel
        .get_chain_info::<Option<CardSet>>(0, CHAININFO_TARGET_CARDS)
        .unwrap_or_default();
    let sg = g.filter(duel, |c, duel| c.is_relate_to_effect(duel, a.e), None);
    duel.remove(sg, POS_FACEUP, REASON_EFFECT)?;
    Ok(())
}
