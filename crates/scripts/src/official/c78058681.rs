// Kewl Tune Synchro
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 78058681;

pub const LISTED_NAMES: &[u32] = &[ID];

// Kewl Tune Mix and Cue for the search, and Mix on the field with Track Maker in the Extra Deck to Synchro Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (16509007, DECK),
    (16509007, DECK),
    (16509007, MZONE),
    (16387555, MZONE),
    (16387555, HAND),
    (42781164, EXTRA),
    (42781164, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::SpecialSummonedCode(42781164)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 2, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_KEWL_TUNE]) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn scfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_TUNER) && c.is_synchro_summonable(duel, None, None)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() && duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? > 0 {
        duel.confirm_cards(1 - tp, g)?;
        duel.shuffle_hand(tp)?;
        let g = duel.get_matching_group(scfilter, tp, LOCATION_EXTRA, 0, None);
        if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 1))? {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            if let Some(sc) = g.select(duel, tp, 1, 1, None)?.get_first() {
                duel.synchro_summon(tp, sc, None, None)?;
            }
        }
    }
    if !e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        return Ok(());
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, splimit);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}

fn splimit(duel: &Duel, _: EffectId, c: CardId) -> bool {
    if c.is_monster(duel) {
        !c.is_type(duel, TYPE_TUNER)
    } else {
        c.is_monster_card(duel) && !c.is_original_type(duel, TYPE_TUNER)
    }
}
