// Big Welcome Labrynth
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, MZONE, OPPONENT, SZONE};

const ID: u32 = 92714517;

pub const LISTED_SERIES: &[u16] = &[SET_LABRYNTH];
// Set, with a copy in the GY to banish, Labrynth monsters (Arias, Arianna) in the Deck to summon and the opponent's Set card to
// return.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, GRAVE),
    (73602965, DECK),
    (73602965, DECK),
    (1225009, DECK),
    (1225009, DECK),
    (1225009, MZONE),
    (44095762, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, ID);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E | TIMING_MAIN_END);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E | TIMING_MAIN_END);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, stock::first_target_to_hand);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_LABRYNTH]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let locations = LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_player_can_sendto_hand(tp)
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, locations, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, locations as i32);
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_MZONE as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) > 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
        let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE, 0, 1, 1, None)?;
        if !g.is_empty() && duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
            let hg = duel.select_matching_card(tp, CardId::is_able_to_hand, tp, LOCATION_MZONE, 0, 1, 1, None)?;
            if !hg.is_empty() {
                duel.hint_selection_full(hg, true);
                duel.break_effect()?;
                duel.send_to_hand(hg, PLAYER_NONE, REASON_EFFECT)?;
            }
        }
    }
    aux::welcome_labrynth_trap_destroy_operation(duel, tp);
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel, tp: u8, fnd_chk: bool) -> bool {
    if !c.is_able_to_hand(duel) {
        return false;
    }
    if c.is_controler(duel, 1 - tp) {
        return fnd_chk;
    }
    c.is_faceup(duel) && c.is_race(duel, RACE_FIEND)
}

fn fndfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_level_above(duel, 8) && c.is_race(duel, RACE_FIEND)
}

// The Lua's target function takes no chkc, so its chkc check reads a nil global and never returns early.
fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let fnd_chk = duel.is_existing_matching_card(fndfilter, tp, LOCATION_MZONE, 0, 1, None);
    let filter = |c: CardId, duel: &Duel| thfilter(c, duel, tp, fnd_chk);
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_MZONE, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_target(tp, filter, tp, LOCATION_MZONE, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, 0, 0);
    Ok(false)
}
