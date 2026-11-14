// Vanquish Soul Trinity Burst
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 53330789;

pub const LISTED_SERIES: &[u16] = &[SET_VANQUISH_SOUL];
// Set, with Vanquish Soul monsters of each Attribute on the field and in the hand to summon, and the opponent's cards to return.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (29302858, MZONE),
    (93156774, MZONE),
    (29280200, HAND),
    (93156774, HAND),
    (29302858, HAND),
    (26202165, MZONE | OPPONENT),
    (53129443, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn tgfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_set_card(duel, &[SET_VANQUISH_SOUL])
        && duel.is_existing_matching_card(
            |sc, duel| spfilter(sc, duel, e, tp, c.get_original_attribute(duel)),
            tp,
            LOCATION_HAND,
            0,
            1,
            None,
        )
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, att: u32) -> bool {
    c.is_set_card(duel, &[SET_VANQUISH_SOUL])
        && c.get_original_attribute(duel) != att
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && tgfilter(chkc, duel, e, tp));
    }
    let filter = |c, duel: &Duel| tgfilter(c, duel, e, tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_target(filter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, filter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, 1 - tp, LOCATION_ONFIELD as i32);
    Ok(false)
}

struct ReturnToHand;

impl aux::DelayedOperation for ReturnToHand {
    fn operate(duel: &mut Duel, rg: CardSet, _: &Args) -> Script<()> {
        duel.send_to_hand(rg, PLAYER_NONE, REASON_EFFECT).map(drop)
    }
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    if !tc.is_relate_to_effect(duel, e) || tc.is_facedown(duel) || ft <= 0 {
        return Ok(());
    }
    let att = tc.get_original_attribute(duel);
    let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp, att), tp, LOCATION_HAND, 0, None);
    if g.is_empty() {
        return Ok(());
    }
    let ct = if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        1
    } else {
        ft.min(2) as usize
    };
    let sg = aux::select_unselect_group(duel, g, 1, ct, &aux::dncheck, tp, HINTMSG_SPSUMMON)?;
    let mut og = CardSet::default();
    for sc in sg {
        if duel.special_summon_step(sc, 0, tp, tp, false, false, POS_FACEUP)? {
            let e1 = duel.create_effect(e.get_handler(duel));
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_DISABLE);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            sc.register_effect(duel, e1);
            let e2 = e1.clone_effect(duel);
            e2.set_code(duel, EFFECT_DISABLE_EFFECT);
            e2.set_value(duel, RESET_TURN_SET as i32);
            sc.register_effect(duel, e2);
            og.insert(sc);
        }
    }
    if duel.special_summon_complete()? == 0 {
        return Ok(());
    }
    if !og.is_empty() {
        aux::delayed_operation_reset::<ReturnToHand>(duel, og, PHASE_END, 0, ID, e, tp);
        let mut thg = CardSet::default();
        for oc in og.union(tc.into()) {
            thg.merge(oc.get_column_group(duel).filter(duel, |c, duel| thfilter(c, duel, 1 - tp), None));
        }
        if !thg.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 1))? {
            duel.break_effect()?;
            duel.send_to_hand(thg, PLAYER_NONE, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel, p: u8) -> bool {
    c.is_able_to_hand(duel) && c.is_controler(duel, p)
}
