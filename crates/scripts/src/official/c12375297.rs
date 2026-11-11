// Elfnote Power Patron
use crate::prelude::*;

const ID: u32 = 12375297;
const JUNORA: u32 = 5914858;

pub const LISTED_NAMES: &[u32] = &[JUNORA];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| {
        c.is_location(duel, LOCATION_EXTRA) && !c.is_synchro_monster(duel)
    });
    c.register_effect(duel, e1);
    aux::add_continuous_lizard_check(duel, c, LOCATION_MZONE, |duel, _, c| !c.is_original_type(duel, TYPE_SYNCHRO));
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_LVCHANGE + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, _| duel.is_main_phase());
    e2.set_target(duel, lvtg);
    e2.set_operation(duel, lvop);
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_BE_MATERIAL);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_location(duel, LOCATION_GRAVE) && a.r == REASON_SYNCHRO
    });
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
}

fn lvfilter(c: CardId, duel: &Duel) -> bool {
    c.is_sequence(duel, 2) && c.has_level(duel) && c.is_faceup(duel)
}

fn lvtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MMZONE) && lvfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(lvfilter, tp, LOCATION_MMZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g = duel.select_target(tp, lvfilter, tp, LOCATION_MMZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_LVCHANGE, g, 1, tp, 3);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn synchrofilter(c: CardId, duel: &Duel) -> bool {
    (c.is_set_card(duel, &[SET_ELFNOTE]) || c.is_code(duel, JUNORA)) && c.is_synchro_summonable(duel, None, None)
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !(tc.is_relate_to_effect(duel, a.e) && tc.is_faceup(duel)) {
        return Ok(());
    }
    let prev_lv = tc.get_level(duel);
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EFFECT_UPDATE_LEVEL);
    e1.set_value(duel, 3);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    tc.register_effect(duel, e1);
    duel.adjust_instantly(Some(tc));
    if !tc.is_level(duel, &[prev_lv + 3]) {
        return Ok(());
    }
    let g = duel.get_matching_group(synchrofilter, tp, LOCATION_EXTRA, 0, None);
    if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        if let Some(sc) = g.select(duel, tp, 1, 1, None)?.get_first() {
            duel.synchro_summon(tp, sc, None, None)?;
        }
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ELFNOTE]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
