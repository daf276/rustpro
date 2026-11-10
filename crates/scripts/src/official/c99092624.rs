// Blooming of the Darkest Rose
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 99092624;

pub const LISTED_NAMES: &[u32] = &[CARD_BLACK_ROSE_DRAGON, TOKEN_ROSE];
// In the hand and Graveyard with a Field Spell (Black Garden) in the Graveyard to count, and a Plant (Angel Trumpeter) on the
// field to banish.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, GRAVE), (71645242, GRAVE), (71645242, HAND), (87979586, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::SpecialSummonedCode(TOKEN_ROSE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_REMOVE + CATEGORY_TODECK + CATEGORY_LEAVE_GRAVE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, tdtg);
    e2.set_operation(duel, tdop);
    c.register_effect(duel, e2);
}

const TOKEN_STATS: MonsterStats = MonsterStats::new(SET_ROSE, TYPES_TOKEN, 800, 800, 2, RACE_PLANT, ATTRIBUTE_DARK);

fn count_field_spells(duel: &Duel, tp: u8) -> usize {
    duel.get_matching_group_count(|c, duel| c.get_sequence(duel) == 5, tp, LOCATION_SZONE, LOCATION_SZONE, None)
        + duel.get_matching_group_count(|c, duel| c.is_type(duel, TYPE_FIELD), tp, LOCATION_GRAVE, LOCATION_GRAVE, None)
}

fn can_summon_token(duel: &mut Duel, tp: u8, p: u8) -> bool {
    duel.get_location_count(p, LOCATION_MZONE) > 0
        && duel.is_player_can_special_summon_monster_full(tp, TOKEN_ROSE, TOKEN_STATS, POS_FACEUP_DEFENSE, p, 0)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        if count_field_spells(duel, tp) == 0 {
            return Ok(false);
        }
        return Ok(can_summon_token(duel, tp, 0) || can_summon_token(duel, tp, 1));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 0, PLAYER_ALL, 0);
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 0, PLAYER_ALL, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let mut ct = count_field_spells(duel, tp);
    if ct == 0 {
        return Ok(());
    }
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ct = 1;
    }
    loop {
        let b1 = can_summon_token(duel, tp, tp);
        let b2 = can_summon_token(duel, tp, 1 - tp);
        if !(b1 || b2) {
            break;
        }
        let op = match (b1, b2) {
            (true, true) => duel.select_option(tp, &[aux::stringid(ID, 1), aux::stringid(ID, 2)])?,
            (true, false) => duel.select_option(tp, &[aux::stringid(ID, 1)])?,
            _ => duel.select_option(tp, &[aux::stringid(ID, 2)])? + 1,
        };
        let p = if op > 0 { 1 - tp } else { tp };
        if let Some(token) = duel.create_token(tp, TOKEN_ROSE) {
            duel.special_summon_step(token, 0, tp, p, false, false, POS_FACEUP_DEFENSE)?;
        }
        ct -= 1;
        if ct == 0 || !duel.select_yes_no(tp, aux::stringid(ID, 0))? {
            break;
        }
    }
    duel.special_summon_complete()?;
    Ok(())
}

fn rmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_able_to_remove(duel) && (c.is_code(duel, CARD_BLACK_ROSE_DRAGON) || c.is_race(duel, RACE_PLANT))
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && rmfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(e.get_handler(duel).is_able_to_deck(duel) && duel.is_existing_target(rmfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, rmfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_TODECK, Some(e.get_handler(duel).into()), 1, 0, 0);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else {
        return Ok(());
    };
    if tc.is_relate_to_effect(duel, e) && duel.remove(tc, 0, REASON_EFFECT | REASON_TEMPORARY)? > 0 {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e1.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
        e1.set_label_object_group(duel, Some(tc.into()));
        e1.set_count_limit(duel, 1, 0);
        if duel.is_phase(PHASE_STANDBY) {
            let turn = i64::from(duel.get_turn_count());
            e1.set_label(duel, turn);
            e1.set_condition(duel, |duel, a| i64::from(duel.get_turn_count()) != a.e.get_label(duel));
            e1.set_reset_full(duel, RESET_PHASE | PHASE_STANDBY, 2);
        } else {
            e1.set_reset(duel, RESET_PHASE | PHASE_STANDBY);
        }
        e1.set_operation(duel, retop);
        duel.register_effect(e1, tp);
        if c.is_relate_to_effect(duel, e) {
            duel.send_to_deck(c, PLAYER_NONE, SEQ_DECKBOTTOM, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn retop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) {
        duel.return_to_field(tc)?;
    }
    Ok(())
}
