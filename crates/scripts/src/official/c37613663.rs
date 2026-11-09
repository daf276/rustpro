// Millennium Ankh
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 37613663;
const EXODIA_INCARNATE: u32 = 83257450;

pub const LISTED_NAMES: &[u32] = &[EXODIA_INCARNATE];

// In hand, with the five limbs of the Forbidden One in the hand and Deck to show, and The Unstoppable Exodia Incarnate in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (EXODIA_INCARNATE, EXTRA),
    (7902349, HAND),
    (8124921, HAND),
    (44519536, DECK),
    (70903634, DECK),
    (7902349, DECK),
    (8124921, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::SpecialSummonedCode(EXODIA_INCARNATE),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TODECK);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, EXODIA_INCARNATE)
        && duel.get_location_count_from_ex(tp, tp, None, c) > 0
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn showfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_FORBIDDEN_ONE])
        && c.is_original_type(duel, TYPE_MONSTER)
        && (c.is_location(duel, LOCATION_HAND | LOCATION_DECK) || c.is_faceup(duel))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(
            duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_EXTRA, 0, 1, None)
                && duel.is_existing_matching_card(showfilter, tp, LOCATION_HAND | LOCATION_DECK | LOCATION_ONFIELD, 0, 5, None),
        );
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    duel.set_operation_info(0, CATEGORY_TODECK, Some(e.get_handler(duel).into()), 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_TODECK, None, 1, tp, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    if c.is_set_card(duel, &[SET_EXODIA]) && c.get_original_level(duel) >= 10 || c.is_set_card(duel, &[SET_MILLENNIUM]) {
        return false;
    }
    c.is_original_type(duel, TYPE_MONSTER) && c.is_faceup(duel) && c.is_able_to_deck(duel)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let mut rg = duel.get_matching_group(showfilter, tp, LOCATION_HAND | LOCATION_DECK | LOCATION_ONFIELD, 0, None);
    if rg.len() > 5 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
        rg = duel.select_matching_card(tp, showfilter, tp, LOCATION_HAND | LOCATION_DECK | LOCATION_ONFIELD, 0, 5, 5, None)?;
    }
    if rg.len() == 5 {
        duel.confirm_cards(1 - tp, rg)?;
        if rg.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_HAND), 1, None) {
            duel.shuffle_hand(tp)?;
        }
        if rg.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_DECK), 1, None) {
            duel.shuffle_deck(tp)?;
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        if let Some(sc) = duel
            .select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_EXTRA, 0, 1, 1, None)?
            .get_first()
        {
            let tdg = duel.get_matching_group(tdfilter, tp, LOCATION_ONFIELD, 0, None);
            duel.break_effect()?;
            if duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP)? > 0 && !tdg.is_empty() {
                duel.break_effect()?;
                duel.send_to_deck(tdg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
            }
        }
    }
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    for code in [EFFECT_CANNOT_FLIP_SUMMON, EFFECT_CANNOT_SPECIAL_SUMMON] {
        let clone = e1.clone_effect(duel);
        clone.set_code(duel, code);
        duel.register_effect(clone, tp);
    }
    if c.is_relate_to_effect(duel, e) && e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        if c.is_has_effect(duel, EFFECT_CANNOT_TO_DECK).is_some() {
            return Ok(());
        }
        c.cancel_to_grave(duel);
        duel.send_to_deck(c, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    }
    Ok(())
}
