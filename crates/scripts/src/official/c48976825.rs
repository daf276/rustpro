// Burial from a Different Dimension
use crate::prelude::*;
use crate::testing::{GRAVE, REMOVED};

const ID: u32 = 48976825;

// Banished monsters of both players to return to the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (CARD_DARK_MAGICIAN, REMOVED),
    (CARD_DARK_MAGICIAN, REMOVED),
    (CARD_DARK_MAGICIAN, GRAVE),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_monster(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_REMOVED) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_REMOVED, LOCATION_REMOVED, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 0));
    let g = duel.select_target(tp, filter, tp, LOCATION_REMOVED, LOCATION_REMOVED, 1, 3, None)?;
    let count = g.map_or(0, |g| g.len() as u8);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, g, count, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let tg = duel
        .get_chain_info::<Option<CardSet>>(0, CHAININFO_TARGET_CARDS)
        .unwrap_or_default();
    let sg = tg.filter(duel, |c, duel| c.is_relate_to_effect(duel, e), None);
    if !sg.is_empty() {
        duel.send_to_grave(sg, REASON_EFFECT | REASON_RETURN)?;
    }
    Ok(())
}
