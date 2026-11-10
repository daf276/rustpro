// Salvage
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND};

const ID: u32 = 96947648;

// In the hand with WATER monsters of 1500 ATK or less (Change Slime, Oppressed People) in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (18914778, GRAVE), (18914778, GRAVE), (58538870, GRAVE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_attack_below(duel, 1500) && c.is_attribute(duel, ATTRIBUTE_WATER) && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_GRAVE, 0, 2, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, filter, tp, LOCATION_GRAVE, 0, 2, 2, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 2, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let g: Option<CardSet> = duel.get_chain_info(0, CHAININFO_TARGET_CARDS);
    let sg = g.unwrap_or_default().filter(duel, |c, duel| c.is_relate_to_effect(duel, a.e), None);
    if !sg.is_empty() {
        duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
