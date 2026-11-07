// De-Fusion
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};

const ID: u32 = 95286165;

// Fusionist, Fusion Summoned by Polymerization with Petit Angel and Mystical Sheep #2, which this then summons back.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (24094653, HAND), (1641882, EXTRA), (38142739, HAND), (83464209, HAND)];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::SpecialSummonedCode(83464209)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TODECK + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_FUSION) && c.is_able_to_extra(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TODECK, g, 1, 0, 0);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn mgfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, fusc: CardId, mg: CardSet) -> bool {
    c.is_controler(duel, tp)
        && c.is_location(duel, LOCATION_GRAVE)
        && c.get_reason(duel) & (REASON_FUSION | REASON_MATERIAL) == REASON_FUSION | REASON_MATERIAL
        && c.get_reason_card(duel) == Some(fusc)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && fusc.check_fusion_material(duel, Some(mg), Some(c.into()), u64::from(PLAYER_NONE) | FUSPROC_NOTFUSION)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else {
        return Ok(());
    };
    if !(tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel)) {
        return Ok(());
    }
    let mg = tc.get_material(duel);
    let ct = mg.len();
    let sumtype = tc.get_summon_type(duel);
    if duel.send_to_deck(tc, PLAYER_NONE, SEQ_DECKTOP, REASON_EFFECT)? != 0
        && sumtype & SUMMON_TYPE_FUSION == SUMMON_TYPE_FUSION
        && ct > 0
        && ct as i32 <= duel.get_location_count(tp, LOCATION_MZONE)
        && mg.filter_count(duel, aux::necro_valley_filter(|c, duel| mgfilter(c, duel, e, tp, tc, mg)), None) == ct
        && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
        && duel.select_yes_no(tp, aux::stringid(ID, 0))?
    {
        duel.break_effect()?;
        duel.special_summon(mg, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
