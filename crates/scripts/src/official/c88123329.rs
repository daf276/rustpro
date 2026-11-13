// Spiritual Beast Rampengu
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE};

const ID: u32 = 88123329;

// On the field with a Ritual Beast Ulti- monster (Cyberse) in the Extra Deck to banish and Ritual Beast Tamer Elder (Cyberse) in the Deck to send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (96113307, EXTRA),
    (96113307, EXTRA),
    (14513016, DECK),
    (14513016, DECK),
    (14513016, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.set_sp_summon_once(duel, ID);
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_REMOVE + CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn tgfilter(c: CardId, duel: &Duel, rac: u64) -> bool {
    c.is_set_card(duel, &[SET_RITUAL_BEAST]) && c.is_race(duel, rac) && c.is_able_to_grave(duel)
}

fn rmfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_RITUAL_BEAST])
        && c.is_able_to_remove(duel)
        && duel.is_existing_matching_card(|tc, duel| tgfilter(tc, duel, c.get_race(duel)), tp, LOCATION_DECK, 0, 1, None)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| rmfilter(c, duel, tp), tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, tp, LOCATION_EXTRA as i32);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, |c, duel| rmfilter(c, duel, tp), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if !g.is_empty() && duel.remove(g, POS_FACEUP, REASON_EFFECT)? != 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
        let race = g.first().map_or(0, |c| c.get_race(duel));
        let sg = duel.select_matching_card(tp, |c, duel| tgfilter(c, duel, race), tp, LOCATION_DECK, 0, 1, 1, None)?;
        duel.send_to_grave(sg, REASON_EFFECT)?;
    }
    Ok(())
}
