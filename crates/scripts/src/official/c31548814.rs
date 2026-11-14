// Void Feast
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 31548814;

pub const LISTED_SERIES: &[u16] = &[SET_VOID, SET_INFERNOID];
// Into the Void in hand to send, with Infernoid Attondel (Level 8), Pirmais (1) and Seitsemas (7) in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (93946239, HAND),
    (93946239, HAND),
    (88301393, DECK),
    (88301393, DECK),
    (60551528, DECK),
    (51316684, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn costfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_VOID])
        && c.is_spell_trap(duel)
        && c.is_able_to_grave_as_cost(duel)
        && (c.is_faceup(duel) || c.is_location(duel, LOCATION_HAND))
        && duel.get_mzone_count(tp, Some(c.into())) > 0
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    e.set_label(duel, 1);
    let handler = e.get_handler(duel);
    let loc = LOCATION_HAND | LOCATION_ONFIELD;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| costfilter(c, duel, tp), tp, loc, 0, 1, Some(handler.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, |c, duel| costfilter(c, duel, tp), tp, loc, 0, 1, 1, Some(handler.into()))?;
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_INFERNOID]) && c.is_monster(duel) && c.is_can_be_special_summoned(duel, e, 0, tp, true, false)
}

fn summon_count(duel: &Duel, tp: u8) -> i32 {
    let ft = duel.get_location_count(tp, LOCATION_MZONE).min(3);
    if ft > 1 && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        1
    } else {
        ft
    }
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let ft = summon_count(duel, tp);
        if e.get_label(duel) == 0 && ft <= 0 {
            return Ok(false);
        }
        e.set_label(duel, 0);
        let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, None);
        return Ok(!g.is_empty() && g.check_with_sum_equal(duel, CardId::get_level, 8, 1, ft as usize));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let ft = summon_count(duel, tp);
    let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, None);
    if g.check_with_sum_equal(duel, CardId::get_level, 8, 1, ft as usize) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = g.select_with_sum_equal(duel, tp, CardId::get_level, 8, 1, ft)?;
        if !sg.is_empty() {
            duel.special_summon(sg, 0, tp, tp, true, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
