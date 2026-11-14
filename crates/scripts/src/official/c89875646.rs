// Triple Borrel Launch
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, OPPONENT};

const ID: u32 = 89875646;

pub const LISTED_SERIES: &[u16] = &[SET_ROKKET];
// In the hand with Dragons and a Field Spell (Zombie World) in the GY, Rokket Caliber in the Deck, and monsters in the opponent's GY.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (75356564, GRAVE),
    (75356564, GRAVE),
    (75356564, GRAVE),
    (4064256, GRAVE),
    (67127799, DECK),
    (67127799, DECK),
    (76704943, GRAVE | OPPONENT),
    (76704943, GRAVE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_DRAGON) && c.is_able_to_deck(duel)
}

fn fieldhandfilter(c: CardId, duel: &Duel) -> bool {
    c.is_field_spell(duel) && c.is_able_to_hand(duel)
}

fn monsterdeckfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_able_to_deck(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let dg = duel.get_matching_group(tdfilter, tp, LOCATION_GRAVE, 0, None);
    let b1 = !duel.has_flag_effect(tp, ID) && !dg.is_empty();
    let b2 = !duel.has_flag_effect(tp, ID + 1)
        && dg.len() >= 2
        && duel.is_existing_matching_card(fieldhandfilter, tp, LOCATION_GRAVE, 0, 1, None);
    let b3 = !duel.has_flag_effect(tp, ID + 2)
        && dg.len() >= 3
        && duel.is_existing_matching_card(monsterdeckfilter, tp, 0, LOCATION_GRAVE, 1, None);
    if chk == 0 {
        return Ok(b1 || b2 || b3);
    }
    let op = duel
        .select_effect(
            tp,
            &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2)), (b3, aux::stringid(ID, 3))],
        )?
        .unwrap_or(0);
    e.set_label(duel, op as i64);
    match op {
        1 => {
            e.set_category(duel, CATEGORY_TODECK + CATEGORY_SPECIAL_SUMMON);
            duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
            duel.set_operation_info(0, CATEGORY_TODECK, None, 1, tp, LOCATION_GRAVE as i32);
            duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
        }
        2 => {
            e.set_category(duel, CATEGORY_TODECK + CATEGORY_TOHAND);
            duel.register_flag_effect(tp, ID + 1, RESET_PHASE | PHASE_END, 0, 1);
            duel.set_operation_info(0, CATEGORY_TODECK, None, 2, tp, LOCATION_GRAVE as i32);
            duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_GRAVE as i32);
        }
        3 => {
            e.set_category(duel, CATEGORY_TODECK);
            duel.register_flag_effect(tp, ID + 2, RESET_PHASE | PHASE_END, 0, 1);
            duel.set_operation_info(0, CATEGORY_TODECK, None, 4, PLAYER_ALL, LOCATION_GRAVE as i32);
        }
        _ => {}
    }
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, code: u32) -> bool {
    c.is_set_card(duel, &[SET_ROKKET]) && !c.is_code(duel, code) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let dg = duel.get_matching_group(tdfilter, tp, LOCATION_GRAVE, 0, None);
    match e.get_label(duel) {
        1 => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
            let Some(gc) = dg.select(duel, tp, 1, 1, None)?.get_first() else {
                return Ok(());
            };
            duel.hint_selection(gc);
            if duel.send_to_deck(gc, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? == 0 || duel.get_location_count(tp, LOCATION_MZONE) <= 0
            {
                return Ok(());
            }
            let code = gc.get_code(duel);
            let sg = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp, code), tp, LOCATION_DECK, 0, None);
            if !sg.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 4))? {
                duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
                let sc = sg.select(duel, tp, 1, 1, None)?;
                duel.break_effect()?;
                duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP)?;
            }
        }
        2 => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
            let g = dg.select(duel, tp, 2, 2, None)?;
            if g.len() != 2 {
                return Ok(());
            }
            duel.hint_selection(g);
            if duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? == 0 {
                return Ok(());
            }
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
            let hg = duel.select_matching_card(tp, fieldhandfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
            if !hg.is_empty() {
                duel.hint_selection(hg);
                duel.break_effect()?;
                duel.send_to_hand(hg, PLAYER_NONE, REASON_EFFECT)?;
            }
        }
        3 => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
            let g = dg.select(duel, tp, 3, 3, None)?;
            if g.len() != 3 {
                return Ok(());
            }
            duel.hint_selection(g);
            if duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? == 0 {
                return Ok(());
            }
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
            let sg = duel.select_matching_card(tp, monsterdeckfilter, tp, 0, LOCATION_GRAVE, 1, 3, None)?;
            if !sg.is_empty() {
                duel.hint_selection(sg);
                duel.break_effect()?;
                duel.send_to_deck(sg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
            }
        }
        _ => {}
    }
    Ok(())
}
