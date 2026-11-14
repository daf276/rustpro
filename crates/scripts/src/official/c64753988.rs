// Cyberdark Realm
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 64753988;

pub const LISTED_SERIES: &[u16] = &[SET_CYBERDARK];
// In the hand and in a random zone, with Cyberdark Claws in the hand to Normal Summon and in the Deck to search, and Cyberdark Wurms.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, 0),
    (82562802, HAND),
    (82562802, DECK),
    (82562802, DECK),
    (56100345, DECK),
    (56100345, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, nstg);
    e2.set_operation(duel, nsop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_CYBERDARK_WORLD);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3.set_target_range(duel, 1, 0);
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_monster(duel)
        && c.is_set_card(duel, &[SET_CYBERDARK])
        && c.is_able_to_hand(duel)
        && !duel.is_existing_matching_card(|gc, duel| gc.is_code(duel, c.get_code(duel)), tp, LOCATION_GRAVE, 0, 1, None)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !a.e.get_handler(duel).is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let g = duel.get_matching_group(|c, duel| thfilter(c, duel, tp), tp, LOCATION_DECK, 0, None);
    if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 0))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let sg = g.select(duel, tp, 1, 1, None)?;
        duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, sg)?;
    }
    Ok(())
}

fn nsfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_CYBERDARK]) && c.is_summonable(duel, true, None)
}

fn nstg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(
            duel.get_location_count(a.tp, LOCATION_MZONE) > 0 && duel.is_existing_matching_card(nsfilter, a.tp, LOCATION_HAND, 0, 1, None)
        );
    }
    duel.set_operation_info(0, CATEGORY_SUMMON, None, 1, 0, 0);
    Ok(false)
}

fn nsop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !a.e.get_handler(duel).is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SUMMON);
    let g = duel.select_matching_card(tp, nsfilter, tp, LOCATION_HAND, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        duel.summon(tp, tc, true, None)?;
    }
    Ok(())
}
