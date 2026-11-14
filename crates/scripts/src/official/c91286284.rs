// Gold Pride - That Came Out of Nowhere!
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND};

const ID: u32 = 91286284;

pub const LISTED_SERIES: &[u16] = &[SET_GOLD_PRIDE];
// In the hand with Gold Pride monsters (Leon, Captain Carrie) in the hand and Graveyard to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (23512906, HAND), (23512906, GRAVE), (96305350, GRAVE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_GOLD_PRIDE]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_GRAVE) as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, 1 - tp, (LOCATION_HAND | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp)),
        tp,
        LOCATION_HAND | LOCATION_GRAVE,
        0,
        1,
        1,
        None,
    )?;
    let opponent_filter = aux::necro_valley_filter(|c: CardId, duel: &Duel| c.is_can_be_special_summoned(duel, e, 0, 1 - tp, false, false));
    if !g.is_empty()
        && duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)? > 0
        && duel.get_location_count(1 - tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(&opponent_filter, 1 - tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, None)
        && duel.select_yes_no(1 - tp, aux::stringid(ID, 1))?
    {
        duel.hint(HINT_SELECTMSG, 1 - tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(1 - tp, &opponent_filter, 1 - tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, 1, None)?;
        let Some(tc) = g.get_first() else { return Ok(()) };
        duel.break_effect()?;
        if duel.special_summon_step(tc, 0, 1 - tp, 1 - tp, false, false, POS_FACEUP)? {
            tc.negate_effects(duel, e.get_handler(duel), 0, false);
        }
        duel.special_summon_complete()?;
    }
    Ok(())
}
