// Magical Dimension
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 28553439;

// In the hand with Spellcasters (Thousand-Eyes Idol, Hurricail) on the field to Tribute and in the hand to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (27125110, MZONE), (27125110, HAND), (15042735, HAND)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_RELEASE + CATEGORY_SPECIAL_SUMMON + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let filter = aux::faceup_filter(|c, duel| c.is_race(duel, RACE_SPELLCASTER));
    duel.is_existing_matching_card(filter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_SPELLCASTER) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn rfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_releasable_by_effect(duel) && duel.get_mzone_count(tp, Some(c.into())) > 0
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && rfilter(chkc, duel, tp));
    }
    if chk == 0 {
        return Ok(
            duel.is_existing_target(|c, duel| rfilter(c, duel, tp), tp, LOCATION_MZONE, 0, 1, None)
                && duel.is_existing_matching_card(|c, duel| filter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, None),
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RELEASE);
    let g = duel.select_target(tp, |c, duel| rfilter(c, duel, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    let dg = duel.get_matching_group(aux::TRUE, tp, LOCATION_MZONE, LOCATION_MZONE, g);
    duel.set_operation_info(0, CATEGORY_RELEASE, g, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, Some(dg), 1, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !(tc.is_relate_to_effect(duel, e)
        && tc.is_controler(duel, tp)
        && duel.release(tc, REASON_EFFECT)? > 0
        && duel.get_location_count(tp, LOCATION_MZONE) > 0)
    {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let sg = duel.select_matching_card(tp, |c, duel| filter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
    if !sg.is_empty() && duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        let dg = duel.get_field_group(tp, LOCATION_MZONE, LOCATION_MZONE);
        if !dg.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 1))? {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
            let des = dg.select(duel, tp, 1, 1, None)?;
            duel.hint_selection_full(des, true);
            duel.break_effect()?;
            duel.destroy(des, REASON_EFFECT)?;
        }
    }
    Ok(())
}
