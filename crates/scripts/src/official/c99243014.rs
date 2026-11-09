// Synchro Overtake
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND};

const ID: u32 = 99243014;

// In the hand, with Junk Warrior in the Extra Deck and Junk Synchron in the Deck and the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (60800381, EXTRA),
    (CARD_JUNK_SYNCHRON, DECK),
    (CARD_JUNK_SYNCHRON, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(CARD_JUNK_SYNCHRON)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    duel.add_custom_activity_counter(
        ID,
        ACTIVITY_SPSUMMON,
        CounterFilter::Card(|duel, c| !c.is_summon_location(duel, LOCATION_EXTRA) || c.is_type(duel, TYPE_SYNCHRO)),
    );
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_custom_activity_count(ID, tp, ACTIVITY_SPSUMMON) == 0);
    }
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| {
        !c.is_type(duel, TYPE_SYNCHRO) && c.is_location(duel, LOCATION_EXTRA)
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| !c.is_original_type(duel, TYPE_SYNCHRO));
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, sc: CardId, ft: i32) -> bool {
    super::material(sc.get_original_code(duel))
        .iter()
        .any(|&code| c.is_code(duel, code))
        && (c.is_able_to_hand(duel) || (ft > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)))
}

fn exfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, ft: i32) -> bool {
    c.is_type(duel, TYPE_SYNCHRO)
        && !super::material(c.get_original_code(duel)).is_empty()
        && duel.is_existing_matching_card(
            |mc, duel| spfilter(mc, duel, e, tp, c, ft),
            tp,
            LOCATION_GRAVE | LOCATION_DECK,
            0,
            1,
            None,
        )
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let ft = duel.get_location_count(tp, LOCATION_MZONE);
        return Ok(duel.is_existing_matching_card(|c, duel| exfilter(c, duel, e, tp, ft), tp, LOCATION_EXTRA, 0, 1, None));
    }
    let location = (LOCATION_DECK | LOCATION_GRAVE) as i32;
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, location);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, location);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
    let cg = duel.select_matching_card(tp, |c, duel| exfilter(c, duel, e, tp, ft), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    let Some(sc) = cg.get_first() else { return Ok(()) };
    duel.confirm_cards(1 - tp, cg)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp, sc, ft));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_GRAVE | LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        aux::to_hand_or_else(
            duel,
            tc,
            tp,
            |tc, duel| tc.is_can_be_special_summoned(duel, e, 0, tp, false, false) && ft > 0,
            |duel| duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP).map(drop),
            aux::stringid(ID, 0),
        )?;
    }
    Ok(())
}
