// Labrynth Set-Up
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 69895264;

pub const LISTED_NAMES: &[u32] = &[ID];
pub const LISTED_SERIES: &[u16] = &[SET_LABRYNTH];

// In the hand, with Welcome Labrynths in the Graveyard to shuffle back, Arias the Labrynth Butler on the field, and Waboku, Trap
// Hole and Raigeki Break in the Deck to Set.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (5380979, GRAVE),
    (5380979, GRAVE),
    (92714517, GRAVE),
    (73602965, MZONE),
    (73602965, MZONE),
    (12607053, DECK),
    (12607053, DECK),
    (4206964, DECK),
    (4178474, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TODECK + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, tdtg);
    e1.set_operation(duel, tdop);
    c.register_effect(duel, e1);
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_LABRYNTH]) && c.is_spell_trap(duel) && !c.is_code(duel, ID) && c.is_able_to_deck(duel)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let locations = LOCATION_GRAVE | LOCATION_REMOVED;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, locations) && tdfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(tdfilter, tp, locations, 0, 2, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, tdfilter, tp, locations, 0, 2, 2, None)?;
    let ct = g.map_or(0, |g| g.len() as u8);
    duel.set_operation_info(0, CATEGORY_TODECK, g, ct, 0, 0);
    Ok(false)
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    !c.is_set_card(duel, &[SET_LABRYNTH]) && c.is_normal_trap(duel) && c.is_ssetable(duel)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if tg.is_empty() || duel.send_to_deck(tg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? < 1 {
        return Ok(());
    }
    let ct = duel
        .get_operated_group()
        .filter_count(duel, |c, duel| c.is_location(duel, LOCATION_DECK), None);
    let fiend = aux::faceup_filter(|c, duel| c.is_race(duel, RACE_FIEND));
    if ct < 1
        || ct as i32 > duel.get_location_count(tp, LOCATION_SZONE)
        || !duel.is_existing_matching_card(fiend, tp, LOCATION_MZONE, 0, 1, None)
    {
        return Ok(());
    }
    let g = duel.get_matching_group(setfilter, tp, LOCATION_DECK, 0, None);
    if aux::select_unselect_group_check(duel, g, ct, ct, &aux::dncheck) && duel.select_yes_no(tp, aux::stringid(ID, 1))? {
        let sg = aux::select_unselect_group(duel, g, ct, ct, &aux::dncheck, tp, HINTMSG_SET)?;
        if !sg.is_empty() {
            duel.break_effect()?;
            duel.sset(tp, sg)?;
        }
    }
    Ok(())
}
