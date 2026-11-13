// Cursed Paper Retribution
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 26116996;
const GROUP_TO_DESTROY: u8 = 0;
const GROUP_TO_NEGATE: u8 = 1;

// Set with monsters on both sides to destroy and negate, and the opponent's Mystical Space Typhoon to send it to the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (ID, SZONE),
    (14214060, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (5318639, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DISABLE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        c.is_previous_location(duel, LOCATION_ONFIELD) && c.is_previous_position(duel, POS_FACEDOWN)
    });
    e2.set_target(duel, negatetg);
    e2.set_operation(duel, negateop);
    c.register_effect(duel, e2);
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, LOCATION_MZONE, 0, 1, None)
            && duel.is_existing_target(CardId::is_negatable, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    let g = duel
        .get_target_group(aux::TRUE, tp, LOCATION_MZONE, 0, None)
        .union(duel.get_target_group(CardId::is_negatable, tp, 0, LOCATION_ONFIELD, None));
    let dpcheck = aux::dpcheck(|c, duel| i64::from(c.get_controler(duel)));
    let tg = aux::select_unselect_group_full(duel, g, 2, 2, &dpcheck, tp, HINTMSG_TARGET, None, false)?;
    duel.set_target_card(tg);
    let (group_to_destroy, group_to_negate) = tg.split(duel, |c, duel| c.is_controler(duel, tp), None);
    e.set_chain_data_group(duel, GROUP_TO_DESTROY, group_to_destroy);
    e.set_chain_data_group(duel, GROUP_TO_NEGATE, group_to_negate);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(group_to_destroy), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_DISABLE, Some(group_to_negate), 1, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tg = duel.get_target_cards(e).unwrap_or_default();
    if tg.is_empty() {
        return Ok(());
    }
    let destroy_tc = tg
        .intersection(e.get_chain_data_group(duel, GROUP_TO_DESTROY).unwrap_or_default())
        .get_first();
    let negate_tc = tg
        .intersection(e.get_chain_data_group(duel, GROUP_TO_NEGATE).unwrap_or_default())
        .get_first();
    if let Some(destroy_tc) = destroy_tc
        && destroy_tc.is_controler(duel, tp)
        && duel.destroy(destroy_tc, REASON_EFFECT)? > 0
        && let Some(negate_tc) = negate_tc
        && negate_tc.is_controler(duel, 1 - tp)
        && negate_tc.is_negatable(duel)
    {
        negate_tc.negate_effects(duel, e.get_handler(duel), RESET_PHASE | PHASE_END, true);
    }
    Ok(())
}

fn negatetg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, 1 - tp) && chkc.is_negatable(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_negatable, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
    let g = duel.select_target(tp, CardId::is_negatable, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DISABLE, g, 1, tp, 0);
    Ok(false)
}

fn negateop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    if let Some(tc) = duel
        .get_first_target()
        .filter(|tc| tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel))
    {
        tc.negate_effects(duel, e.get_handler(duel), 0, true);
    }
    Ok(())
}
