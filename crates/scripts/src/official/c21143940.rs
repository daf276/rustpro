// Mask Change
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 21143940;

// Destiny HERO - Malicious (a DARK HERO) on the field and Masked HERO Dark Law in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (9411399, MZONE), (58481572, EXTRA)];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(58481572)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_END_PHASE);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn tfilter(c: CardId, duel: &Duel, att: u32, e: EffectId, tp: u8, mc: CardId) -> bool {
    c.is_set_card(duel, &[SET_MASKED_HERO])
        && c.is_attribute(duel, att)
        && duel.get_location_count_from_ex(tp, tp, Some(mc.into()), c) > 0
        && c.is_can_be_special_summoned(duel, e, 0, tp, true, false)
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_set_card(duel, &[SET_HERO])
        && duel.is_existing_matching_card(
            |tc, duel| tfilter(tc, duel, c.get_attribute(duel), e, tp, c),
            tp,
            LOCATION_EXTRA,
            0,
            1,
            None,
        )
}

fn chkfilter(c: CardId, duel: &Duel, att: u32) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_HERO]) && c.is_attribute(duel, att)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp)
            && chkc.is_location(duel, LOCATION_MZONE)
            && chkfilter(chkc, duel, e.get_label(duel) as u32));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| filter(c, duel, e, tp), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_target(tp, |c, duel| filter(c, duel, e, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    e.set_label(
        duel,
        g.and_then(|g| g.get_first()).map_or(0, |tc| i64::from(tc.get_attribute(duel))),
    );
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let att = tc.get_attribute(duel);
    if duel.send_to_grave(tc, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let g = duel.get_matching_group(|c, duel| tfilter(c, duel, att, e, tp, tc), tp, LOCATION_EXTRA, 0, None);
    if g.is_empty() {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let sg = g.select(duel, tp, 1, 1, None)?;
    if let Some(sc) = sg.get_first() {
        duel.break_effect()?;
        duel.special_summon(sg, 0, tp, tp, true, false, POS_FACEUP)?;
        sc.complete_procedure(duel);
    }
    Ok(())
}
