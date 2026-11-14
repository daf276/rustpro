// Runick Tip
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};
use engine::constants::MSG_CONFIRM_CARDS;

const ID: u32 = 31562086;

pub const LISTED_SERIES: &[u16] = &[SET_RUNICK];
pub const LISTED_NAMES: &[u32] = &[ID];

struct Quickplay;

impl MysteruneQuickPlay for Quickplay {
    const ID: u32 = ID;
    const UNIQUECAT: u64 = CATEGORY_SEARCH + CATEGORY_TOHAND;
    const RMCOUNT: usize = 1;

    fn uniquetg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
        thtg(duel, a, chk)
    }

    fn uniqueop(duel: &mut Duel, a: &Args) -> Script<bool> {
        thop(duel, a)
    }
}

// Hugin the Runick Wings in the Extra Deck to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (55990317, EXTRA)];

pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(55990317), Check::Message(MSG_CONFIRM_CARDS)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Mysterune::create_qp_effect::<Quickplay>(duel, c);
    c.register_effect(duel, e1);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RUNICK]) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<bool> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first()
        && duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? > 0
        && tc.is_location(duel, LOCATION_HAND)
    {
        duel.confirm_cards(1 - tp, g)?;
        return Ok(true);
    }
    Ok(false)
}
