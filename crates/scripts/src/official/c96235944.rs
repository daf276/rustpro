// Void Breach
use crate::prelude::*;
use crate::testing::{Check, HAND, REMOVED, SZONE};

use super::c60830240::not_faceup_fiend;

const ID: u32 = 96235944;

pub const LISTED_SERIES: &[u16] = &[SET_INFERNOID];
// Face-up in the Spell & Trap Zone with Infernoids banished to add and in hand to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (60551528, REMOVED),
    (58446973, REMOVED),
    (88301393, HAND),
    (23440231, HAND),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_INFERNOID]));
    e2.set_value(
        duel,
        Value::Card(|duel, e, _| {
            100 * duel.get_matching_group_count(atkfilter, e.get_handler_player(duel), LOCATION_REMOVED, 0, None) as i32
        }),
    );
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_condition(duel, |duel, a| {
        duel.get_matching_group_count(not_faceup_fiend, a.tp, LOCATION_MZONE, 0, None) == 0
    });
    e3.set_target(duel, target);
    e3.set_operation(duel, operation);
    c.register_effect(duel, e3);
}

fn atkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_INFERNOID]) && c.is_monster(duel) && c.is_faceup(duel)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_monster(duel) && c.is_set_card(duel, &[SET_INFERNOID]) && c.is_can_be_special_summoned(duel, e, 0, tp, true, false)
}

fn tohafilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_set_card(duel, &[SET_INFERNOID]) && c.is_faceup(duel) && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let b1 = duel.is_existing_matching_card(tohafilter, tp, LOCATION_REMOVED, 0, 1, None);
    let b2 = duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        e.set_category(duel, CATEGORY_TOHAND);
        duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_REMOVED as i32);
    } else if op == Some(2) {
        e.set_category(duel, CATEGORY_SPECIAL_SUMMON);
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    }
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let op = e.get_label(duel);
    if op == 1 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let g = duel.select_matching_card(tp, tohafilter, tp, LOCATION_REMOVED, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, g)?;
        }
    } else if op == 2 {
        if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
            return Ok(());
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.special_summon(g, 0, tp, tp, true, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
