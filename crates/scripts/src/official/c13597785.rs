// Elfnote Lucina
use crate::prelude::*;

const ID: u32 = 13597785;

pub const LISTED_NAMES: &[u32] = &[ID];

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
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_TOHAND);
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

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ELFNOTE]) && c.is_monster(duel) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, a.tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}

fn swtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(!a.e.get_handler(duel).is_sequence(duel, 2) && duel.get_field_card(a.tp, LOCATION_MZONE, 2).is_some());
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, 1 - a.tp, LOCATION_MZONE as i32);
    Ok(false)
}

fn rthfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level_below(duel, 6) && c.is_faceup(duel) && c.is_able_to_hand(duel)
}

fn swop(duel: &mut Duel, a: &Args) -> Script<()> {
    swap_then_return(duel, a, ID, rthfilter, 0, LOCATION_MZONE)
}

// The swap the Elfnote swap effects start with; whether it happened.
pub(crate) fn swap(duel: &mut Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let cc = duel.get_field_card(a.tp, LOCATION_MZONE, 2);
    c.is_relate_to_effect(duel, a.e) && c.is_in_main_mzone(duel, Some(a.tp)) && cc.is_some_and(|cc| c != cc && duel.swap_sequence(c, cc))
}

pub(crate) fn swap_then_return(duel: &mut Duel, a: &Args, id: u32, filter: fn(CardId, &Duel) -> bool, s: u32, o: u32) -> Script<()> {
    let tp = a.tp;
    if !swap(duel, a) {
        return Ok(());
    }
    let g = duel.get_matching_group(filter, tp, s, o, None);
    if g.is_empty() || !duel.select_yes_no(tp, aux::stringid(id, 3))? {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let sg = g.select(duel, tp, 1, 1, None)?;
    if !sg.is_empty() {
        duel.hint_selection(sg);
        duel.break_effect()?;
        duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
