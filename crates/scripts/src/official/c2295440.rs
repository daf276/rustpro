// One for One
use crate::prelude::*;
use crate::testing::{DECK, HAND};

const ID: u32 = 2295440;

// Peacock in the hand to send, and Key Mace, a Level 1 Normal Monster, in the hand and Deck to Summon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (20624263, HAND), (1929294, DECK), (1929294, DECK), (1929294, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn costfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_monster(duel)
        && c.is_able_to_grave_as_cost(duel)
        && duel.is_existing_matching_card(
            |c, duel| filter(c, duel, e, tp),
            tp,
            LOCATION_HAND | LOCATION_DECK,
            0,
            1,
            Some(c.into()),
        )
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.get_level(duel) == 1 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn cost(duel: &mut Duel, a: &Args, _: i32) -> Script<bool> {
    a.e.set_label(duel, 1);
    Ok(true)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
            return Ok(false);
        }
        if e.get_label(duel) != 0 {
            e.set_label(duel, 0);
            return Ok(duel.is_existing_matching_card(|c, duel| costfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, None));
        }
        return Ok(duel.is_existing_matching_card(|c, duel| filter(c, duel, e, tp), tp, LOCATION_HAND | LOCATION_DECK, 0, 1, None));
    }
    if e.get_label(duel) != 0 {
        e.set_label(duel, 0);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
        let g = duel.select_matching_card(tp, |c, duel| costfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
        duel.send_to_grave(g, REASON_COST)?;
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_DECK) as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        |c, duel| filter(c, duel, e, tp),
        tp,
        LOCATION_HAND | LOCATION_DECK,
        0,
        1,
        1,
        None,
    )?;
    duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    Ok(())
}
