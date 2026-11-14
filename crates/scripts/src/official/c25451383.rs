// Albion the Shrouded Dragon
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND};

const ID: u32 = 25451383;

pub const LISTED_SERIES: &[u16] = &[SET_BRANDED];
pub const LISTED_NAMES: &[u32] = &[CARD_ALBAZ];

// In the hand and the Graveyard, with Fallen of Albaz in the hand to Special Summon it and in the Deck to shuffle it into the Deck, and Branded Spells in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (CARD_ALBAZ, HAND),
    (CARD_ALBAZ, DECK),
    (29948294, DECK),
    (44362883, DECK),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_CHANGE_CODE);
    e1.set_range(duel, LOCATION_MZONE | LOCATION_GRAVE);
    e1.set_value(duel, CARD_ALBAZ as i32);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_HAND | LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, spcost);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    if !((c.is_code(duel, CARD_ALBAZ) || (c.is_set_card(duel, &[SET_BRANDED]) && c.is_spell_trap(duel)))
        && c.is_able_to_grave_as_cost(duel))
    {
        return false;
    }
    let hc = e.get_handler(duel);
    if c.is_location(duel, LOCATION_HAND) {
        duel.get_location_count_full(tp, LOCATION_MZONE, 0, LOCATION_REASON_TOFIELD, 0xff) > 0
            && hc.is_can_be_special_summoned(duel, e, 0, tp, false, false)
    } else {
        hc.is_able_to_deck(duel) && (hc.is_location(duel, LOCATION_GRAVE) || duel.is_player_can_draw(tp, 1))
    }
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let filter = |c, duel: &Duel| cfilter(c, duel, e, tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_HAND | LOCATION_DECK, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let Some(tc) = duel
        .select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_DECK, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(false);
    };
    e.set_label(duel, i64::from(tc.is_location(duel, LOCATION_DECK)));
    duel.send_to_grave(tc, REASON_COST)?;
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let (e, c, tp) = (a.e, a.e.get_handler(duel), a.tp);
    if e.get_label(duel) == 0 {
        e.set_category(duel, CATEGORY_SPECIAL_SUMMON);
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    } else {
        let mut cat = CATEGORY_TODECK;
        if c.is_location(duel, LOCATION_HAND) {
            cat += CATEGORY_DRAW;
            duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
        }
        e.set_category(duel, cat);
        duel.set_operation_info(0, CATEGORY_TODECK, Some(c.into()), 1, tp, 0);
    }
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c, tp) = (a.e, a.e.get_handler(duel), a.tp);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    if e.get_label(duel) == 0 && duel.get_location_count_full(tp, LOCATION_MZONE, 0, LOCATION_REASON_TOFIELD, 0xff) > 0 {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    } else {
        let loc = c.get_location(duel);
        if duel.send_to_deck(c, PLAYER_NONE, SEQ_DECKBOTTOM, REASON_EFFECT)? > 0
            && c.is_location(duel, LOCATION_DECK)
            && loc == LOCATION_HAND
        {
            duel.break_effect()?;
            duel.draw(tp, 1, REASON_EFFECT)?;
        }
    }
    Ok(())
}
