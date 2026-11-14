// The Fallen & The Virtuous
use crate::prelude::*;

const ID: u32 = 30271097;

pub const LISTED_SERIES: &[u16] = &[SET_BRANDED, SET_DOGMATIKA, SET_ECCLESIA];

// The field of its chain data table.
const CHOICE: u8 = 0;

pub const LISTED_NAMES: &[u32] = &[CARD_ALBAZ];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_cost(duel, effcost);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    e1.set_hint_timing(
        duel,
        TIMING_END_PHASE,
        TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E,
    );
    c.register_effect(duel, e1);
}

fn descostfilter(c: CardId, duel: &Duel) -> bool {
    c.lists_code(duel, CARD_ALBAZ) && c.is_able_to_grave_as_cost(duel)
}

fn spconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ECCLESIA]) && c.is_monster(duel) && c.is_faceup(duel)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

// b1 and b2, which the cost and the target both check.
fn choices(duel: &Duel, e: EffectId, tp: u8) -> (bool, bool) {
    let c = e.get_handler(duel);
    let b1 = duel.is_existing_matching_card(descostfilter, tp, LOCATION_EXTRA, 0, 1, None)
        && duel.is_existing_target(
            |tc, duel| tc.is_faceup(duel),
            tp,
            LOCATION_ONFIELD,
            LOCATION_ONFIELD,
            1,
            Some(c.into()),
        );
    let b2 = duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(spconfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, None)
        && duel.is_existing_target(|tc, duel| spfilter(tc, duel, e, tp), tp, LOCATION_GRAVE, LOCATION_GRAVE, 1, None);
    (b1, b2)
}

fn effcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let (b1, b2) = choices(duel, e, tp);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let choice = duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?;
    if let Some(choice) = choice {
        e.set_chain_data(duel, CHOICE, choice as i64);
    }
    if choice == Some(1) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
        let g = duel.select_matching_card(tp, descostfilter, tp, LOCATION_EXTRA, 0, 1, 1, None)?;
        duel.send_to_grave(g, REASON_COST)?;
    }
    Ok(false)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(chkc) = chkc {
        // Without a choice the Lua falls through to the checks below, whose chk 0 answer is the result.
        match e.get_chain_data(duel, CHOICE) {
            Some(1) => return Ok(chkc.is_on_field(duel) && chkc.is_faceup(duel) && chkc != c),
            Some(2) => return Ok(chkc.is_location(duel, LOCATION_GRAVE) && spfilter(chkc, duel, e, tp)),
            _ => {}
        }
    }
    let (b1, b2) = choices(duel, e, tp);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let choice = match e.get_chain_data(duel, CHOICE) {
        Some(choice) => Some(choice as usize),
        None => duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?,
    };
    if let Some(choice) = choice {
        e.set_chain_data(duel, CHOICE, choice as i64);
    }
    if choice == Some(1) {
        e.set_category(duel, CATEGORY_DESTROY);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let g = duel.select_target(
            tp,
            |tc, duel| tc.is_faceup(duel),
            tp,
            LOCATION_ONFIELD,
            LOCATION_ONFIELD,
            1,
            1,
            Some(c.into()),
        )?;
        duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    } else if choice == Some(2) {
        e.set_category(duel, CATEGORY_SPECIAL_SUMMON);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_target(
            tp,
            |tc, duel| spfilter(tc, duel, e, tp),
            tp,
            LOCATION_GRAVE,
            LOCATION_GRAVE,
            1,
            1,
            None,
        )?;
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    match e.get_chain_data(duel, CHOICE) {
        Some(1) => {
            duel.destroy(tc, REASON_EFFECT)?;
        }
        Some(2) => {
            duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
        }
        _ => {}
    }
    Ok(())
}
