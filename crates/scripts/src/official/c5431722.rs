// Dracotail Flame
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, OPPONENT, SZONE};

const ID: u32 = 5431722;

pub const LISTED_SERIES: &[u16] = &[SET_DRACOTAIL];
// Spells on the field to negate, and a Dracotail monster in the Graveyard to place on the bottom of the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (ID, GRAVE),
    (75003700, GRAVE),
    (75003700, DECK),
    (24094653, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Resolves];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE + CATEGORY_TODECK + CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn disfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell(duel) && c.is_negatable_spell_trap(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && disfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(disfilter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
    let g = duel.select_target(tp, disfilter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DISABLE, g, 1, tp, 0);
    place_and_draw_info(duel, tp);
    Ok(false)
}

pub fn place_and_draw_info(duel: &mut Duel, tp: u8) {
    duel.set_possible_operation_info(0, CATEGORY_TODECK, None, 1, tp, (LOCATION_GRAVE | LOCATION_REMOVED) as i32);
    duel.set_possible_operation_info(0, CATEGORY_DRAW, None, 1, tp, 1);
}

pub fn tdfilter<const ID: u32>(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DRACOTAIL]) && c.is_faceup(duel) && c.is_able_to_deck(duel) && !c.is_code(duel, ID)
}

// The part every Dracotail Spell/Trap ends with: place a Dracotail card on the bottom of the Deck, then draw 1 card.
pub fn place_and_draw<const ID: u32>(duel: &mut Duel, tp: u8) -> Script<()> {
    let filter = aux::necro_valley_filter(tdfilter::<ID>);
    let locations = LOCATION_GRAVE | LOCATION_REMOVED;
    if !(duel.is_existing_matching_card(&filter, tp, locations, 0, 1, None)
        && duel.is_player_can_draw(tp, 1)
        && duel.select_yes_no(tp, aux::stringid(ID, 1))?)
    {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_matching_card(tp, &filter, tp, locations, 0, 1, 1, None)?;
    if g.is_empty() {
        return Ok(());
    }
    duel.hint_selection(g);
    duel.break_effect()?;
    if duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKBOTTOM, REASON_EFFECT)? > 0 {
        duel.break_effect()?;
        duel.draw(tp, 1, REASON_EFFECT)?;
    }
    Ok(())
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else {
        return Ok(());
    };
    if tc.is_faceup(duel) && tc.is_relate_to_effect(duel, e) && tc.is_can_be_disabled_by_effect(duel, e, true) {
        tc.negate_effects(duel, e.get_handler(duel), RESETS_STANDARD_PHASE_END, false);
        duel.adjust_instantly(Some(tc));
        if tc.is_disabled(duel) {
            place_and_draw::<ID>(duel, tp)?;
        }
    }
    Ok(())
}
