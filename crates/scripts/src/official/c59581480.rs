// Elfnote Tinia
use super::c13597785::swap;
use crate::prelude::*;

const ID: u32 = 59581480;

pub const LISTED_SERIES: &[u16] = &[SET_ELFNOTE];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_value(duel, Value::Values(&[0, ZONE_CENTER_MMZ]));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, pltg);
    e2.set_operation(duel, plop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_REMOVE);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_MMZONE);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e3.set_target(duel, swtg);
    e3.set_operation(duel, swop);
    e3.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e3);
}

fn plfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_ELFNOTE]) && c.is_continuous_spell(duel) && !c.is_forbidden(duel) && c.check_unique_on_field(duel, tp)
}

fn pltg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_matching_card(|c, duel| plfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    Ok(false)
}

fn plop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.get_location_count(tp, LOCATION_SZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    let sc = duel.select_matching_card(tp, |c, duel| plfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(sc) = sc.get_first() {
        duel.move_to_field(sc, tp, tp, LOCATION_SZONE, POS_FACEUP, true)?;
    }
    Ok(())
}

fn swtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(!a.e.get_handler(duel).is_sequence(duel, 2) && duel.get_field_card(a.tp, LOCATION_MZONE, 2).is_some());
    }
    duel.set_possible_operation_info(0, CATEGORY_REMOVE, None, 1, 1 - a.tp, LOCATION_HAND as i32);
    Ok(false)
}

fn swop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !swap(duel, a) {
        return Ok(());
    }
    let g = duel.get_matching_group(|c, duel| c.is_able_to_remove(duel), tp, 0, LOCATION_HAND, None);
    if g.is_empty() || !duel.select_yes_no(tp, aux::stringid(ID, 3))? {
        return Ok(());
    }
    let rg = g.random_select(duel, tp, 1);
    if !rg.is_empty() {
        duel.break_effect()?;
        aux::remove_until::<ReturnToHand>(duel, rg, POS_FACEUP, REASON_EFFECT | REASON_TEMPORARY, PHASE_END, ID, a.e, tp)?;
    }
    Ok(())
}

struct ReturnToHand;

impl aux::DelayedOperation for ReturnToHand {
    fn operate(duel: &mut Duel, ag: CardSet, _: &Args) -> Script<()> {
        duel.send_to_hand(ag, PLAYER_NONE, REASON_EFFECT)?;
        Ok(())
    }
}
