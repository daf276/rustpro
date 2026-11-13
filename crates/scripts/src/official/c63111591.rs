// Corpse-Seller Crow
use crate::prelude::*;
use crate::testing::{DECK, SZONE};

const ID: u32 = 63111591;

pub const LISTED_NAMES: &[u32] = &[ID];

// Set, with Normal Monsters (Kozaky) and Spells (Pot of Greed) in the Deck to mill, then Special Summon or Set.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (99171160, DECK),
    (99171160, DECK),
    (99171160, DECK),
    (55144522, DECK),
    (55144522, DECK),
    (55144522, DECK),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_SPECIAL_SUMMON + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_discard_deck(tp, 1));
    }
    duel.set_operation_info(0, CATEGORY_DECKDES, None, 0, tp, 1);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_GRAVE as i32);
    duel.set_possible_operation_info(0, CATEGORY_SET, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel) && c.is_ssetable(duel) && !c.is_code(duel, ID)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if !duel.is_player_can_discard_deck(tp, 1) {
        return Ok(());
    }
    let trap_count = duel
        .get_matching_group(CardId::is_trap, tp, LOCATION_GRAVE, LOCATION_GRAVE, None)
        .get_class_count(duel, |c, duel| i64::from(c.get_code(duel)))
        + 1;
    let deck_count = duel.get_field_group_count(tp, LOCATION_DECK, 0);
    let max_count = trap_count.min(deck_count).min(4);
    let mut mill_count = 1;
    if max_count > 1 {
        mill_count = duel.announce_number_range(tp, 1, max_count as u32)?;
    }
    if duel.discard_deck(tp, mill_count as u16, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let mut og = duel.get_operated_group();
    og.keep_matching(duel, |c, duel| c.is_location(duel, LOCATION_GRAVE), None);
    if og.is_empty() {
        return Ok(());
    }
    let spfilter = |c: CardId, duel: &Duel| c.is_can_be_special_summoned(duel, e, 0, tp, false, false);
    let b1 = duel.get_location_count(tp, LOCATION_MZONE) > 0 && og.is_exists(duel, aux::necro_valley_filter(spfilter), 1, None);
    let b2 = og.is_exists(duel, aux::necro_valley_filter(setfilter), 1, None);
    if !((b1 || b2) && duel.select_yes_no(tp, aux::stringid(ID, 1))?) {
        return Ok(());
    }
    let Some(op) = duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])? else {
        return Ok(());
    };
    if op == 1 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = og.filter_select(duel, tp, aux::necro_valley_filter(spfilter), 1, 1, None)?;
        if !g.is_empty() {
            duel.break_effect()?;
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    } else if op == 2 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
        let Some(sc) = og
            .filter_select(duel, tp, aux::necro_valley_filter(setfilter), 1, 1, None)?
            .get_first()
        else {
            return Ok(());
        };
        duel.break_effect()?;
        if duel.sset(tp, sc)? > 0 && (sc.is_trap(duel) || sc.is_quick_play_spell(duel)) {
            let eff_code = if sc.is_trap(duel) {
                EFFECT_TRAP_ACT_IN_SET_TURN
            } else {
                EFFECT_QP_ACT_IN_SET_TURN
            };
            let e1 = duel.create_effect(e.get_handler(duel));
            e1.set_description(duel, aux::stringid(ID, 4));
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_property(duel, EFFECT_FLAG_SET_AVAILABLE + EFFECT_FLAG_CLIENT_HINT);
            e1.set_code(duel, eff_code);
            e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
            sc.register_effect(duel, e1);
        }
    }
    Ok(())
}
