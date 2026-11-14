// Trickstar Crimson Heart
use engine::constants::MSG_RECOVER;

use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 51011872;

pub const LISTED_SERIES: &[u16] = &[SET_TRICKSTAR];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_TRICKSTAR], Some(lc), sumtype, tp)
    }
}

// In the Extra Deck, Link Summoned with two Trickstar Nightshades, with more in the hand to discard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (86825114, MZONE),
    (86825114, MZONE),
    (86825114, HAND),
    (86825114, HAND),
];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::Message(MSG_RECOVER)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, reccon);
    e1.set_operation(duel, recop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_DRAW);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_cost(duel, drcost);
    e3.set_target(duel, drtg);
    e3.set_operation(duel, drop);
    c.register_effect(duel, e3);
}

fn cfilter(c: CardId, duel: &Duel, lg: CardSet) -> bool {
    c.is_set_card(duel, &[SET_TRICKSTAR]) && lg.is_contains(c)
}

fn reccon(duel: &Duel, a: &Args) -> bool {
    let lg = a.e.get_handler(duel).get_linked_group(duel);
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| cfilter(c, duel, lg), 1, None))
}

fn recop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.hint(HINT_CARD, 0, ID.into());
    duel.recover(a.tp, 200, REASON_EFFECT)?;
    Ok(())
}

fn costfilter(c: CardId, duel: &Duel) -> bool {
    c.is_discardable(duel) && c.is_set_card(duel, &[SET_TRICKSTAR])
}

fn drcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(costfilter, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, costfilter, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let ct = if duel.get_lp(tp) >= duel.get_lp(1 - tp) + 2000 { 2 } else { 1 };
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, ct) && duel.is_player_can_draw(1 - tp, 1));
    }
    a.e.set_label(duel, ct as i64);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, PLAYER_ALL, 1);
    Ok(false)
}

fn drop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.draw(tp, a.e.get_label(duel) as u32, REASON_EFFECT)?;
    duel.draw(1 - tp, 1, REASON_EFFECT)?;
    Ok(())
}
