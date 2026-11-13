// Comic Cat
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 72921536;

pub const LISTED_NAMES: &[u32] = &[CARD_TOON_WORLD];

// On the field with Toon World in the hand and Toon World monsters in the hand and Deck to Special Summon, and a monster of the
// opponent to Tribute.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (CARD_TOON_WORLD, HAND),
    (CARD_TOON_WORLD, HAND),
    (45536531, HAND),
    (45536531, DECK),
    (8915275, DECK),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(45536531)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_ADD_TYPE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, _| {
        duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_code(duel, CARD_TOON_WORLD)),
            0,
            LOCATION_ONFIELD,
            LOCATION_ONFIELD,
            1,
            None,
        )
    });
    e1.set_value(duel, TYPE_TOON as i32);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_RELEASE + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, _| duel.is_main_phase());
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    c.register_effect(duel, e2);
}

fn tribfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_releasable_by_effect(duel) && duel.get_mzone_count(tp, Some(c.into())) > 0
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.lists_code(duel, CARD_TOON_WORLD) && c.is_monster(duel) && c.is_can_be_special_summoned(duel, e, 0, tp, true, false)
}

fn opp_location(duel: &Duel, tp: u8) -> u32 {
    let toon_world = aux::faceup_filter(|c, duel| c.is_code(duel, CARD_TOON_WORLD));
    if duel.is_existing_matching_card(toon_world, tp, LOCATION_ONFIELD, 0, 1, None) {
        LOCATION_MZONE
    } else {
        0
    }
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(
            |c, duel| tribfilter(c, duel, tp),
            tp,
            LOCATION_MZONE,
            opp_location(duel, tp),
            1,
            None,
        ) && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND | LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_RELEASE, None, 1, PLAYER_EITHER, LOCATION_MZONE as i32);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_DECK) as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RELEASE);
    let opp_location = opp_location(duel, tp);
    let rg = duel.select_matching_card(tp, |c, duel| tribfilter(c, duel, tp), tp, LOCATION_MZONE, opp_location, 1, 1, None)?;
    if rg.is_empty() {
        return Ok(());
    }
    duel.hint_selection(rg);
    if duel.release(rg, REASON_EFFECT)? > 0 && duel.get_location_count(tp, LOCATION_MZONE) > 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = duel.select_matching_card(
            tp,
            |c, duel| spfilter(c, duel, e, tp),
            tp,
            LOCATION_HAND | LOCATION_DECK,
            0,
            1,
            1,
            None,
        )?;
        if !sg.is_empty() {
            duel.special_summon(sg, 0, tp, tp, true, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
