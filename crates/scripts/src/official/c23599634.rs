// Artmage Pact -Awakening-
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 23599634;

pub const LISTED_SERIES: &[u16] = &[SET_ARTMAGE];
pub const LISTED_NAMES: &[u32] = &[CARD_MEDIUS_THE_PURE];

// In hand to activate and in the Graveyard to return Artmage Litera, with a second one in the Deck to Special Summon and a monster of
// the opponent to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (97434754, DECK),
    (97434754, MZONE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_TOEXTRA + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    (c.is_set_card(duel, &[SET_ARTMAGE]) || c.is_code(duel, CARD_MEDIUS_THE_PURE))
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) > 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    let c = e.get_handler(duel);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 2)));
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_INACTIVATE);
    e1.set_value(duel, Value::Chain(efilter));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_condition(duel, limcon);
    e2.set_operation(duel, limop);
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e2, tp);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EVENT_CHAIN_END);
    e3.set_operation(duel, limop2);
    e3.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e3, tp);
    Ok(())
}

fn efilter(duel: &Duel, e: EffectId, ct: u8) -> bool {
    let p = e.get_handler_player(duel);
    let tp: u8 = duel.get_chain_info(ct, CHAININFO_TRIGGERING_PLAYER);
    duel.get_chain_info::<Option<EffectId>>(ct, CHAININFO_TRIGGERING_EFFECT)
        .is_some_and(|te| p == tp && te.is_has_category(duel, CATEGORY_FUSION_SUMMON))
}

fn limfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_summon_player(duel, tp) && c.is_summon_type(duel, &[SUMMON_TYPE_FUSION])
}

fn limcon(duel: &Duel, a: &Args) -> bool {
    a.eg.unwrap_or_default()
        .is_exists(duel, |c, duel| limfilter(c, duel, a.tp), 1, None)
}

fn chainlm(_: &Duel, _: EffectId, rp: u8, tp: u8, _: i64) -> bool {
    tp == rp
}

fn limop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    match duel.get_current_chain() {
        0 => duel.set_chain_limit_till_chain_end(chainlm, 0),
        1 => {
            let c = e.get_handler(duel);
            c.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
            e1.set_code(duel, EVENT_CHAINING);
            e1.set_operation(duel, resetop);
            duel.register_effect(e1, tp);
            let e2 = e1.clone_effect(duel);
            e2.set_code(duel, EVENT_BREAK_EFFECT);
            e2.set_reset(duel, RESET_CHAIN);
            duel.register_effect(e2, tp);
        }
        _ => {}
    }
    Ok(())
}

fn resetop(duel: &mut Duel, a: &Args) -> Script<()> {
    a.e.get_handler(duel).reset_flag_effect(duel, ID);
    a.e.reset(duel);
    Ok(())
}

fn limop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.has_flag_effect(duel, ID) {
        duel.set_chain_limit_till_chain_end(chainlm, 0);
    }
    c.reset_flag_effect(duel, ID);
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ARTMAGE]) && c.is_faceup(duel) && (c.is_able_to_hand(duel) || c.is_able_to_extra(duel))
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && thfilter(chkc, duel));
    }
    let dg = duel.get_field_group(tp, 0, LOCATION_ONFIELD);
    if chk == 0 {
        return Ok(!dg.is_empty() && duel.is_existing_target(thfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(dg), 1, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)? > 0
        && tc.is_location(duel, LOCATION_HAND | LOCATION_EXTRA)
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let g = duel.select_matching_card(tp, aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
        if !g.is_empty() {
            duel.hint_selection(g);
            duel.destroy(g, REASON_EFFECT)?;
        }
    }
    Ok(())
}
