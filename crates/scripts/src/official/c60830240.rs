// Void Reignition
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, REMOVED};

const ID: u32 = 60830240;

pub const LISTED_SERIES: &[u16] = &[SET_INFERNOID, SET_VOID];
// Infernoids in hand to reveal and banished to return, with a copy in the GY.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (60551528, HAND),
    (58446973, HAND),
    (60551528, REMOVED),
    (26034577, REMOVED),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_HANDES + CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_cost(duel, dccost);
    e1.set_target(duel, dctg);
    e1.set_operation(duel, dcop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_DUEL);
    e2.set_condition(duel, tgcon);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, tgtg);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
}

pub fn not_faceup_fiend(c: CardId, duel: &Duel) -> bool {
    !(c.is_faceup(duel) && c.is_race(duel, RACE_FIEND))
}

fn tgcon(duel: &Duel, a: &Args) -> bool {
    aux::exccon(duel, a) && !duel.is_existing_matching_card(not_faceup_fiend, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn dccostfilter(c: CardId, duel: &Duel) -> bool {
    !c.is_public(duel)
        && ((c.is_set_card(duel, &[SET_INFERNOID]) && c.is_monster(duel)) || (c.is_set_card(duel, &[SET_VOID]) && c.is_spell_trap(duel)))
}

fn dccost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(dccostfilter, tp, LOCATION_HAND, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
    let g = duel.select_matching_card(tp, dccostfilter, tp, LOCATION_HAND, 0, 1, 1, Some(c.into()))?;
    duel.confirm_cards(1 - tp, g)?;
    duel.shuffle_hand(tp)?;
    Ok(false)
}

fn dctg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_field_group(tp, LOCATION_HAND, 0).difference(a.e.get_handler(duel).into());
    let ct = g.len();
    if chk == 0 {
        return Ok(ct > 0 && duel.is_player_can_draw(tp, ct));
    }
    duel.set_operation_info(0, CATEGORY_HANDES, Some(g), ct as u8, 0, 0);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, ct as i32);
    Ok(false)
}

fn dcop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_field_group(tp, LOCATION_HAND, 0);
    if g.is_empty() {
        return Ok(());
    }
    let ct = duel.send_to_grave(g, REASON_EFFECT | REASON_DISCARD)?;
    if ct > 0 && duel.is_player_can_draw(tp, 0) {
        duel.break_effect()?;
        duel.draw(tp, ct as u32, REASON_EFFECT)?;
    }
    Ok(())
}

fn tgfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_faceup(duel) && c.is_monster(duel) && c.is_set_card(duel, &[SET_INFERNOID]) && c.is_can_be_effect_target(duel, e)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let g = duel.get_matching_group(|c, duel| tgfilter(c, duel, e), tp, LOCATION_REMOVED, 0, None);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    let tg = aux::select_unselect_group(duel, g, 1, 11, &aux::dncheck, tp, aux::stringid(ID, 2))?;
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, Some(tg), tg.len() as u8, 0, 0);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    if !g.is_empty() {
        duel.send_to_grave(g, REASON_EFFECT | REASON_RETURN)?;
    }
    Ok(())
}
