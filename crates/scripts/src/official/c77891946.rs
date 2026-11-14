// Exosister Vadis
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, SZONE};

const ID: u32 = 77891946;

pub const LISTED_SERIES: &[u16] = &[SET_EXOSISTER];

// Set and in the hand, with Exosister Sophia and Irene (which list each other) in the Deck to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, HAND),
    (5352328, DECK),
    (5352328, DECK),
    (79858629, DECK),
    (79858629, DECK),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(5352328), Check::SpecialSummonedCode(79858629)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_cost(duel, Cost::pay_lp::<800>);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_EXOSISTER]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn rescon(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    (sg.is_exists(duel, |c, duel| lfilter1(c, duel, sg), 1, None), false)
}

fn lfilter1(c: CardId, duel: &Duel, sg: CardSet) -> bool {
    sg.is_exists(duel, |sc, duel| sc.lists_code(duel, c.get_code(duel)), 1, Some(c.into()))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let mg = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, None);
    if chk == 0 {
        return Ok(duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && duel.get_location_count(tp, LOCATION_MZONE) > 1
            && aux::select_unselect_group_check(duel, mg, 2, 2, &rescon));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, 0, LOCATION_DECK as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() || duel.get_location_count(tp, LOCATION_MZONE) < 2 {
        return Ok(());
    }
    let mg = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, None);
    if mg.len() < 2 {
        return Ok(());
    }
    let g1 = aux::select_unselect_group(duel, mg, 2, 2, &rescon, tp, HINTMSG_SPSUMMON)?;
    if g1.len() > 1 {
        let fid = c.get_field_id(duel).into();
        for tc in g1 {
            duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP)?;
            tc.register_flag_effect_full(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1, fid, 0);
        }
        duel.special_summon_complete()?;
        let e1 = duel.create_effect(c);
        e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
        e1.set_category(duel, CATEGORY_TODECK);
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e1.set_code(duel, EVENT_PHASE + PHASE_END);
        e1.set_count_limit(duel, 1, 0);
        e1.set_label(duel, fid);
        e1.set_label_object_group(duel, Some(g1));
        e1.set_condition_mut(duel, tdcon);
        e1.set_operation(duel, tdop);
        duel.register_effect(e1, tp);
    }
    if e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, aux::stringid(ID, 1));
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| {
            !c.is_set_card(duel, &[SET_EXOSISTER]) && c.is_location(duel, LOCATION_EXTRA)
        });
        e1.set_target_range(duel, 1, 0);
        duel.register_effect(e1, tp);
        aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| !c.is_original_set_card(duel, &[SET_EXOSISTER]));
    }
    Ok(())
}

fn tdfilter(c: CardId, duel: &Duel, fid: i64) -> bool {
    c.get_flag_effect_label(duel, ID) == Some(fid)
}

fn tdcon(duel: &mut Duel, a: &Args) -> bool {
    let g = a.e.get_label_object_group(duel).unwrap_or_default();
    let fid = a.e.get_label(duel);
    if !g.is_exists(duel, |c, duel| tdfilter(c, duel, fid), 1, None) {
        a.e.reset(duel);
        return false;
    }
    true
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let fid = a.e.get_label(duel);
    let tg =
        a.e.get_label_object_group(duel)
            .unwrap_or_default()
            .filter(duel, |c, duel| tdfilter(c, duel, fid), None);
    duel.send_to_deck(tg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    Ok(())
}
