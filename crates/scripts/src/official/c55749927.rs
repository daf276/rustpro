// Horn of Olifant
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, MZONE, SZONE};

const ID: u32 = 55749927;

pub const LISTED_SERIES: &[u16] = &[SET_ROLAND];
// Burstinatrix (a Level 3 FIRE Warrior) in the Deck to summon three of for Roland (a Synchro, destroyed), and Durendal in the Graveyard to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (ID, SZONE),
    (40251688, MZONE),
    (58932615, DECK),
    (58932615, DECK),
    (58932615, DECK),
    (58932615, DECK),
    (37478723, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummonedCode(58932615),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_END_PHASE | TIMINGS_CHECK_MONSTER);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, rmvtg);
    e1.set_operation(duel, rvmop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_hint_timing(duel, 0, TIMING_END_PHASE | TIMINGS_CHECK_MONSTER);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_spell(duel) && c.is_type(duel, TYPE_EQUIP) && c.is_able_to_remove(duel)
}

fn rmvtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_ONFIELD | LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, tp, (LOCATION_ONFIELD | LOCATION_GRAVE) as i32);
    duel.set_operation_info(0, CATEGORY_DESTROY, None, 1, tp, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn rvmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_ONFIELD | LOCATION_GRAVE, 0, 1, 1, None)?;
    if duel.remove(g, POS_FACEUP, REASON_EFFECT)? > 0 {
        let dg = duel.get_matching_group(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
        if !dg.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
            duel.break_effect()?;
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
            let tc = dg.select(duel, tp, 1, 1, None)?;
            duel.hint_selection(tc);
            duel.destroy(tc, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn desfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_ROLAND]) && c.is_destructable_full(duel, e)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_attribute(duel, ATTRIBUTE_FIRE)
        && c.is_race(duel, RACE_WARRIOR)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, None);
    let ct = duel.get_location_count_full(tp, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, 0xff);
    if chk == 0 {
        return Ok(
            duel.is_existing_matching_card(|c, duel| desfilter(c, duel, e), tp, LOCATION_MZONE, 0, 1, None)
                && g.check_with_sum_equal(duel, CardId::get_level, 9, 1, ct.max(0) as usize),
        );
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_matching_card(tp, |c, duel| desfilter(c, duel, e), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    if duel.destroy(g, REASON_EFFECT)? > 0 {
        let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
        if ft == 0 {
            return Ok(());
        }
        let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, None);
        if !g.is_empty() {
            ft = ft.min(3);
            if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
                ft = 1;
            }
            if g.check_with_sum_equal(duel, CardId::get_level, 9, 1, ft as usize) {
                duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
                let sg = g.select_with_sum_equal(duel, tp, CardId::get_level, 9, 1, ft)?;
                for tc in sg {
                    if duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)? {
                        for code in [EFFECT_DISABLE, EFFECT_DISABLE_EFFECT] {
                            let e1 = duel.create_effect(e.get_handler(duel));
                            e1.set_type(duel, EFFECT_TYPE_SINGLE);
                            e1.set_code(duel, code);
                            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
                            tc.register_effect_full(duel, e1, true);
                        }
                    }
                }
                duel.special_summon_complete()?;
            }
        }
    }
    let e0 = duel.create_effect(e.get_handler(duel));
    e0.set_type(duel, EFFECT_TYPE_FIELD);
    e0.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e0.set_description(duel, aux::stringid(ID, 3));
    e0.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    if duel.is_turn_player(tp) {
        e0.set_reset_full(duel, RESET_PHASE | PHASE_END | RESET_SELF_TURN, 2);
    } else {
        e0.set_reset(duel, RESET_PHASE | PHASE_END | RESET_SELF_TURN);
    }
    e0.set_target_range(duel, 1, 0);
    e0.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| !c.is_race(duel, RACE_WARRIOR));
    duel.register_effect(e0, tp);
    Ok(())
}
