// Asceticism of the Six Samurai
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 27821104;

pub const LISTED_SERIES: &[u16] = &[SET_SIX_SAMURAI];
// In the hand beside Legendary Six Samurai - Kageki, with Chamberlain of the Six Samurai (same ATK) in the Deck to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (2511717, MZONE), (44430454, DECK), (44430454, DECK), (44430454, DECK)];
pub const CHECKS: &[Check] = &[Check::Resolves];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn tfilter(c: CardId, duel: &Duel, atk: i32, code: u32, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_SIX_SAMURAI])
        && !c.is_code(duel, code)
        && c.get_attack(duel) == atk
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    let (atk, code) = (c.get_attack(duel), c.get_code(duel));
    c.is_faceup(duel)
        && c.is_set_card(duel, &[SET_SIX_SAMURAI])
        && duel.is_existing_matching_card(|c, duel| tfilter(c, duel, atk, code, e, tp), tp, LOCATION_DECK, 0, 1, None)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && filter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| filter(c, duel, e, tp), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, |c, duel| filter(c, duel, e, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    let tc = duel.get_first_target();
    unported!(duel, tc.is_none());
    let Some(tc) = tc else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, e) || tc.is_facedown(duel) {
        return Ok(());
    }
    let (atk, code) = (tc.get_attack(duel), tc.get_code(duel));
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let sg = duel.select_matching_card(tp, |c, duel| tfilter(c, duel, atk, code, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !sg.is_empty() {
        duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EVENT_PHASE + PHASE_END);
    e1.set_operation(duel, desop);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    e1.set_count_limit(duel, 1, 0);
    tc.register_effect(duel, e1);
    Ok(())
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.destroy(a.e.get_handler(duel), REASON_EFFECT)?;
    Ok(())
}
