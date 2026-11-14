// Exosister Martha
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, OPPONENT};

const ID: u32 = 37343995;

pub const LISTED_SERIES: &[u16] = &[SET_EXOSISTER];
pub const LISTED_NAMES: &[u32] = &[16474916];

// In the hand with Exosister Elis in the Deck to summon, and the opponent's Monster Reborn to take a monster out of a Graveyard,
// for Exosister Karmael in the Extra Deck to be Xyz Summoned.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (16474916, DECK),
    (16474916, DECK),
    (77675029, EXTRA),
    (77675029, EXTRA),
    (83764718, HAND | OPPONENT),
    (83764718, HAND | OPPONENT),
    (15023985, GRAVE),
    (15023985, GRAVE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, hspcon);
    e1.set_cost(duel, hspcost);
    e1.set_target(duel, hsptg);
    e1.set_operation(duel, hspop);
    c.register_effect(duel, e1);
    duel.add_custom_activity_counter(
        ID,
        ACTIVITY_SPSUMMON,
        CounterFilter::Card(|duel, c| c.is_set_card(duel, &[SET_EXOSISTER])),
    );
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_LEAVE_GRAVE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, super::c16474916::sptg);
    e2.set_operation(duel, super::c16474916::spop);
    c.register_effect(duel, e2);
}

fn hspcon(duel: &Duel, a: &Args) -> bool {
    !duel.is_existing_matching_card(
        |c, duel| !aux::faceup_filter(|c, duel| c.is_type(duel, TYPE_XYZ))(c, duel),
        a.tp,
        LOCATION_MZONE,
        0,
        1,
        None,
    )
}

fn hspcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_custom_activity_count(ID, tp, ACTIVITY_SPSUMMON) == 0);
    }
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH + EFFECT_FLAG_CLIENT_HINT);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| !c.is_set_card(duel, &[SET_EXOSISTER]));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(false)
}

fn dspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, 16474916) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn hsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && duel.get_location_count(tp, LOCATION_MZONE) > 1
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && duel.is_existing_matching_card(|c, duel| dspfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(
        0,
        CATEGORY_SPECIAL_SUMMON,
        Some(c.into()),
        2,
        tp,
        (LOCATION_DECK | LOCATION_HAND) as i32,
    );
    Ok(false)
}

fn hspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        return Ok(());
    }
    if duel.special_summon_step(c, 0, tp, tp, false, false, POS_FACEUP)? && duel.get_location_count(tp, LOCATION_MZONE) > 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |c, duel| dspfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
        if let Some(sc) = g.get_first() {
            duel.special_summon_step(sc, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    duel.special_summon_complete()?;
    Ok(())
}
