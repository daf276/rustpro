// ALERT!
use crate::prelude::*;
use crate::testing::{GRAVE, HAND};

const ID: u32 = 26372118;

pub const LISTED_SERIES: &[u16] = &[SET_RESCUE_ACE];
pub const LISTED_NAMES: &[u32] = &[CARD_R_ACE_HYDRANT];
// In the hand, with Rescue-ACE Impulse in the Graveyard to add.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (38339996, GRAVE), (38339996, GRAVE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_END_PHASE);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RESCUE_ACE]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn hydrant_chk(duel: &Duel, tp: u8) -> bool {
    let filter = aux::faceup_filter(|c, duel| c.is_code(duel, CARD_R_ACE_HYDRANT));
    duel.is_existing_matching_card(filter, tp, LOCATION_ONFIELD, 0, 1, None)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let mut loc = LOCATION_GRAVE;
    if hydrant_chk(duel, tp) {
        loc |= LOCATION_DECK;
    }
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, loc, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, (LOCATION_GRAVE | LOCATION_DECK) as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let mut loc = LOCATION_GRAVE;
    if hydrant_chk(duel, tp) {
        loc |= LOCATION_DECK;
    }
    stock::search(duel, a, aux::necro_valley_filter(thfilter), loc)
}
