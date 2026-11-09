// The Book of the Law
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 458748;

// In the hand twice, with Invoked Raidjin (WIND) on the field to Tribute and Invoked Sorath, Caliga and Oceanus in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (49513164, MZONE),
    (43989315, EXTRA),
    (13529466, EXTRA),
    (6772168, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_label(duel, 0);
    e1.set_cost(duel, |duel, a, chk| {
        a.e.set_label(duel, 100);
        Ok(chk == 0)
    });
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn cfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_INVOKED])
        && duel.is_existing_matching_card(
            |sc, duel| spfilter(sc, duel, e, tp, c.get_original_attribute(duel), Some(c)),
            tp,
            LOCATION_EXTRA,
            0,
            1,
            None,
        )
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, att: u32, mc: Option<CardId>) -> bool {
    duel.get_location_count_from_ex(tp, tp, mc.map(CardSet::from), c) > 0
        && c.is_set_card(duel, &[SET_INVOKED])
        && !c.is_original_attribute(duel, att)
        && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_FUSION, tp, false, false)
        && c.check_fusion_material(duel, None, None, u64::from(PLAYER_NONE))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        if e.get_label(duel) != 100 {
            return Ok(false);
        }
        e.set_label(duel, 0);
        return Ok(duel.check_release_group_cost(tp, |c, duel| cfilter(c, duel, e, tp), (1, 1), false, None, None));
    }
    let rg = duel.select_release_group_cost(tp, |c, duel| cfilter(c, duel, e, tp), (1, 1), false, None, None)?;
    if let Some(tc) = rg.get_first() {
        let att = tc.get_original_attribute(duel);
        e.set_label(duel, i64::from(att));
    }
    duel.release(rg, REASON_COST)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let att = e.get_label(duel) as u32;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp, att, None), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    tc.set_material(duel, None);
    if duel.special_summon(tc, SUMMON_TYPE_FUSION, tp, tp, false, false, POS_FACEUP)? > 0 {
        tc.complete_procedure(duel);
    }
    Ok(())
}
