// Mimighoul Room
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, SZONE};

const ID: u32 = 59293853;

pub const LISTED_SERIES: &[u16] = &[SET_MIMIGHOUL];
// Set copies and one in the Graveyard, with Mimighoul Fairy in the hand and Deck to Special Summon and on the field to return.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (ID, GRAVE),
    (43066927, HAND),
    (43066927, DECK),
    (43066927, DECK),
    (43066927, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_POSITION + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, f1: bool, f2: bool) -> bool {
    if !c.is_set_card(duel, &[SET_MIMIGHOUL]) {
        return false;
    }
    (f1 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false))
        || (f2 && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEDOWN_DEFENSE, 1 - tp, 0xff))
}

fn zones(duel: &Duel, tp: u8) -> (bool, bool) {
    (
        duel.get_location_count(tp, LOCATION_MZONE) > 0,
        duel.get_location_count_full(1 - tp, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, 0xff) > 0,
    )
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let (f1, f2) = zones(duel, tp);
        return Ok(duel.is_existing_matching_card(
            |c, duel| spfilter(c, duel, e, tp, f1, f2),
            tp,
            LOCATION_HAND | LOCATION_DECK,
            0,
            1,
            None,
        ));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_DECK) as i32);
    duel.set_possible_operation_info(0, CATEGORY_POSITION, None, 1, PLAYER_EITHER, POS_FACEDOWN_DEFENSE as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let (f1, f2) = zones(duel, tp);
    if !(f1 || f2) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = |c, duel: &Duel| spfilter(c, duel, e, tp, f1, f2);
    let Some(tc) = duel
        .select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_DECK, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    let b1 = f1 && tc.is_can_be_special_summoned(duel, e, 0, tp, false, false);
    let b2 = f2 && tc.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEDOWN_DEFENSE, 1 - tp, 0xff);
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?;
    let (target_player, summon_pos) = if op == Some(1) {
        (tp, POS_FACEUP)
    } else {
        (1 - tp, POS_FACEDOWN_DEFENSE)
    };
    if duel.special_summon(tc, 0, tp, target_player, false, false, summon_pos)? == 0 {
        return Ok(());
    }
    if tc.is_facedown(duel) {
        duel.confirm_cards(tp, tc)?;
    }
    let g = duel.get_matching_group(CardId::is_can_turn_set, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    if g.is_empty() || !duel.select_yes_no(tp, aux::stringid(ID, 4))? {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
    if let Some(pc) = g.select(duel, tp, 1, 1, None)?.get_first() {
        duel.hint_selection(pc);
        duel.break_effect()?;
        duel.change_position(pc, POS_FACEDOWN_DEFENSE)?;
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_MIMIGHOUL]) && c.is_able_to_hand(duel) && c.is_can_be_effect_target(duel, e)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_ONFIELD) && thfilter(chkc, duel, e));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| thfilter(c, duel, e), tp, LOCATION_ONFIELD, 0, 1, None));
    }
    let g = duel.get_matching_group(|c, duel| thfilter(c, duel, e), tp, LOCATION_ONFIELD, 0, None);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let tg = g.select(duel, tp, 1, g.len() as u8, None)?;
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(tg), tg.len() as u8, tp, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if !tg.is_empty() {
        duel.send_to_hand(tg, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
