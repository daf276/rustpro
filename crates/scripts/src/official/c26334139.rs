// Sinful Spoils Subdual
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, SZONE};

const ID: u32 = 26334139;

// In the hand and Set, with Level 5 or higher Illusions (Diabellze the White Witch) in the Deck, and Set Mirror Walls to return.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (ID, SZONE),
    (60145298, DECK),
    (60145298, DECK),
    (60145298, DECK),
    (22359980, SZONE),
    (22359980, SZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, ID);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMING_BATTLE_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_target(duel, thsptg);
    e1.set_operation(duel, thspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMING_BATTLE_END | TIMINGS_CHECK_MONSTER_E);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn thspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, ft: i32) -> bool {
    c.is_race(duel, RACE_ILLUSION)
        && c.is_level_above(duel, 5)
        && (c.is_able_to_hand(duel) || (ft > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)))
}

fn thsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let ft = duel.get_location_count(tp, LOCATION_MZONE);
        return Ok(duel.is_existing_matching_card(|c, duel| thspfilter(c, duel, e, tp, ft), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
    let filter = |c, duel: &Duel| thspfilter(c, duel, e, tp, ft);
    let Some(sc) = duel.select_matching_card(tp, filter, tp, LOCATION_DECK, 0, 1, 1, None)?.get_first() else {
        return Ok(());
    };
    aux::to_hand_or_else(
        duel,
        sc,
        tp,
        |sc, duel| ft > 0 && sc.is_can_be_special_summoned(duel, e, 0, tp, false, false),
        |duel| {
            if duel.special_summon_step(sc, 0, tp, tp, false, false, POS_FACEUP)? {
                let e1 = duel.create_effect(e.get_handler(duel));
                e1.set_description(duel, aux::stringid(ID, 3));
                e1.set_type(duel, EFFECT_TYPE_SINGLE);
                e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
                e1.set_code(duel, EFFECT_CANNOT_TRIGGER);
                e1.set_label(duel, i64::from(tp));
                e1.set_condition(duel, |duel, a| {
                    duel.is_main_phase() && a.e.get_handler(duel).is_controler(duel, a.e.get_label(duel) as u8)
                });
                e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
                sc.register_effect(duel, e1);
            }
            duel.special_summon_complete()?;
            Ok(())
        },
        aux::stringid(ID, 4),
    )
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_hand(duel) && c.is_facedown(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, tp) && chkc.is_able_to_hand(duel) && chkc.is_facedown(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter, tp, LOCATION_ONFIELD, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_ONFIELD, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, tp, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, a.e) || duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    duel.shuffle_hand(tp)?;
    let g = duel.get_matching_group(CardId::is_ssetable, tp, LOCATION_HAND, 0, None);
    if g.is_empty() || !duel.select_yes_no(tp, aux::stringid(ID, 5))? {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    if let Some(sc) = g.select(duel, tp, 1, 1, None)?.get_first() {
        duel.break_effect()?;
        duel.sset_full(tp, sc, tp, false)?;
    }
    Ok(())
}
