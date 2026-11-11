// Swordsoul of Taia
use super::c20001443::{drcon, tkop, tktg};
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, MZONE};

const ID: u32 = 56495147;
const SWORDSOUL_OF_MO_YE: u32 = 20001443;
const SWORDSOUL_GRANDMASTER_CHIXIAO: u32 = 69248256;
const THE_GOLDEN_SWORDSOUL: u32 = 82489470;

pub const LISTED_NAMES: &[u32] = &[TOKEN_SWORDSOUL];

// On the field and in the Graveyard with Swordsoul of Mo Ye to banish, the Golden Swordsoul as the Tuner and in the Deck to
// send, and Swordsoul Grandmaster - Chixiao in the Extra Deck to Synchro Summon with it (which sends Taia to the Graveyard).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (SWORDSOUL_OF_MO_YE, GRAVE),
    (THE_GOLDEN_SWORDSOUL, MZONE),
    (THE_GOLDEN_SWORDSOUL, DECK),
    (SWORDSOUL_GRANDMASTER_CHIXIAO, EXTRA),
    (SWORDSOUL_GRANDMASTER_CHIXIAO, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummonedCode(TOKEN_SWORDSOUL),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, tkcost);
    e1.set_target(duel, tktg);
    e1.set_operation(duel, tkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_BE_MATERIAL);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, drcon);
    e2.set_target(duel, tgtg);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SWORDSOUL]) || (c.is_monster(duel) && c.is_race(duel, RACE_WYRM))
}

fn tkcostfilter(c: CardId, duel: &Duel) -> bool {
    filter(c, duel) && c.is_able_to_remove_as_cost(duel) && aux::sp_elim_filter(c, duel, true, false)
}

fn tkcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let locations = LOCATION_MZONE | LOCATION_GRAVE;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(tkcostfilter, tp, locations, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, tkcostfilter, tp, locations, 0, 1, 1, None)?;
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_grave(duel) && filter(c, duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, tgfilter, LOCATION_DECK, CATEGORY_TOGRAVE)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::send_to_grave(duel, a, tgfilter, LOCATION_DECK)
}
