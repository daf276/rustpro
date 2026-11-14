// Runick Fountain
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND};

const ID: u32 = 92107604;

pub const LISTED_SERIES: &[u16] = &[SET_RUNICK];
// Activated from the hand, then a Runick Quick-Play Spell from the hand with Runick Tips in the Graveyard to return.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (31562086, HAND),
    (31562086, HAND),
    (31562086, GRAVE),
    (31562086, GRAVE),
    (55990317, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_QP_ACT_IN_NTPHAND);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_target_range(duel, LOCATION_HAND, 0);
    e2.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_RUNICK]));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_TODECK + CATEGORY_DRAW);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_CHAINING);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, drcon);
    e3.set_target(duel, drtg);
    e3.set_operation(duel, drop);
    c.register_effect(duel, e3);
}

fn drcon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    a.rp == a.tp
        && re.get_handler(duel).is_set_card(duel, &[SET_RUNICK])
        && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)
        && re.is_spell_effect(duel)
        && re.is_active_type(duel, TYPE_QUICKPLAY)
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RUNICK]) && c.is_quick_play_spell(duel) && c.is_able_to_deck(duel)
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && tdfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(tdfilter, tp, LOCATION_GRAVE, 0, 1, None) && duel.is_player_can_draw(tp, 0));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, tdfilter, tp, LOCATION_GRAVE, 0, 1, 3, None)?;
    let count = g.map_or(0, |g| g.len()) as u8;
    duel.set_operation_info(0, CATEGORY_TODECK, g, count, 0, 0);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, i32::from(count));
    Ok(false)
}

fn drop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    if g.is_empty() {
        return Ok(());
    }
    let count = duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKBOTTOM, REASON_EFFECT)?;
    if count == 0 {
        return Ok(());
    }
    if count > 1 {
        duel.sort_deckbottom(tp, tp, count as u16)?;
    }
    duel.break_effect()?;
    duel.draw(tp, count as u32, REASON_EFFECT)?;
    Ok(())
}
