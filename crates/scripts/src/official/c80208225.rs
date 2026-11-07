// Dracotail Sting
use super::c5431722::{place_and_draw, place_and_draw_info};
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, OPPONENT};

const ID: u32 = 80208225;

pub const LISTED_NAMES: &[u32] = &[ID];

// A monster and a Spell/Trap in the opponent's Graveyard to banish, and a Dracotail monster in the Graveyard to place on the
// bottom of the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (24094653, GRAVE | OPPONENT),
    (13597785, GRAVE | OPPONENT),
    (75003700, GRAVE),
    (75003700, DECK),
];
pub const CHECKS: &[Check] = &[Check::Resolves];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE + CATEGORY_TODECK + CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
}

fn rmfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_able_to_remove(duel) && c.is_can_be_effect_target(duel, e)
}

fn rescon(duel: &Duel, sg: CardSet) -> bool {
    sg.filter_count(duel, CardId::is_monster, None) <= 1 && sg.filter_count(duel, CardId::is_spell_trap, None) <= 1
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_able_to_remove(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_able_to_remove, tp, 0, LOCATION_GRAVE, 1, None));
    }
    let g = duel.get_matching_group(|c, duel| rmfilter(c, duel, e), tp, 0, LOCATION_GRAVE, None);
    let tg = aux::select_unselect_group(duel, g, 1, 2, &|duel, sg, _, _| (rescon(duel, sg), false), tp, HINTMSG_REMOVE)?;
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(tg), tg.len() as u8, tp, 0);
    place_and_draw_info(duel, tp);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if !tg.is_empty() && duel.remove(tg, POS_FACEUP, REASON_EFFECT)? > 0 {
        place_and_draw::<ID>(duel, tp)?;
    }
    Ok(())
}
