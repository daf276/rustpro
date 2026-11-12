// Infernoid Decatron
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 58446973;

pub const LISTED_NAMES: &[u32] = &[ID];

// Copies in the hand to Normal Summon, and Infernoid Pirmais, a monster with effects to copy, in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (60551528, DECK), (60551528, DECK)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_INFERNOID]) && c.is_monster(duel) && !c.is_code(duel, ID) && c.is_able_to_grave(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, filter, LOCATION_DECK, CATEGORY_TOGRAVE)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    let c = a.e.get_handler(duel);
    if let Some(tc) = g.get_first()
        && duel.send_to_grave(tc, REASON_EFFECT)? != 0
        && tc.is_location(duel, LOCATION_GRAVE)
        && c.is_relate_to_effect(duel, a.e)
        && c.is_faceup(duel)
    {
        let code = tc.get_code(duel);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_LEVEL);
        e1.set_value(duel, tc.get_level(duel) as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_code(duel, EFFECT_CHANGE_CODE);
        e2.set_value(duel, code as i32);
        e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e2);
        c.copy_effect(duel, code, RESET_EVENT | RESETS_STANDARD, 1);
    }
    Ok(())
}
