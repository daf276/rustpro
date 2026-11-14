// Swordsoul Grandmaster - Chixiao
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE, OPPONENT};

const ID: u32 = 69248256;

pub const LISTED_SERIES: &[u16] = &[SET_SWORDSOUL];

const SWORDSOUL_OF_MO_YE: u32 = 20001443;
const THE_GOLDEN_SWORDSOUL: u32 = 82489470;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp) && c.is_race_full(duel, RACE_WYRM, Some(sc), sumtype, tp)
    }
}

// In the Extra Deck, Synchro Summoned from several Golden Swordsouls and Mo Yes on the field, with Mo Ye in the Deck
// to search and banish, and an opponent's Mo Ye to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (THE_GOLDEN_SWORDSOUL, MZONE),
    (THE_GOLDEN_SWORDSOUL, MZONE),
    (THE_GOLDEN_SWORDSOUL, MZONE),
    (THE_GOLDEN_SWORDSOUL, MZONE),
    (SWORDSOUL_OF_MO_YE, MZONE),
    (SWORDSOUL_OF_MO_YE, MZONE),
    (SWORDSOUL_OF_MO_YE, DECK),
    (SWORDSOUL_OF_MO_YE, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e1.set_target(duel, thrmtg);
    e1.set_operation(duel, thrmop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DISABLE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e2.set_cost(duel, discost);
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
}

fn thrmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SWORDSOUL]) && (c.is_able_to_hand(duel) || c.is_able_to_remove(duel))
}

fn thrmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thrmfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_REMOVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thrmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
    let Some(sc) = duel
        .select_matching_card(tp, thrmfilter, tp, LOCATION_DECK, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    aux::to_hand_or_else(
        duel,
        sc,
        tp,
        |sc, duel| sc.is_able_to_remove(duel),
        |duel| duel.remove(sc, POS_FACEUP, REASON_EFFECT).map(drop),
        aux::stringid(ID, 3),
    )
}

fn discostfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_set_card(duel, &[SET_SWORDSOUL]) || c.is_race(duel, RACE_WYRM))
        && c.is_able_to_remove_as_cost(duel)
        && (c.is_location(duel, LOCATION_HAND) || aux::sp_elim_filter(c, duel, true, false))
}

fn discost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let locations = LOCATION_HAND | LOCATION_GRAVE | LOCATION_MZONE;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(discostfilter, tp, locations, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, discostfilter, tp, locations, 0, 1, 1, None)?;
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn disfilter(c: CardId, duel: &Duel) -> bool {
    c.is_negatable_monster(duel) && c.is_type(duel, TYPE_EFFECT)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && disfilter(chkc, duel) && chkc != c);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(disfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
    let g = duel.select_target(tp, disfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, Some(c.into()))?;
    duel.set_operation_info(0, CATEGORY_DISABLE, g, 1, tp, 0);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_faceup(duel) && tc.is_relate_to_effect(duel, a.e) {
        tc.negate_effects(duel, a.e.get_handler(duel), RESET_PHASE | PHASE_END, false);
    }
    Ok(())
}
