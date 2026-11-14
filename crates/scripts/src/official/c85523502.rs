// Whisker Blitzclique
use super::c22912101::{register_effect_monster_lock, spfilter};
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 85523502;

pub const LISTED_SERIES: &[u16] = &[SET_BLITZCLIQUE];
// In the hand and on the field, with Thunder Normal Monsters (LaLa Li-oon) in the hand and on the field, and Surge
// Blitzclique in the hand to destroy a card with.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (9430387, HAND),
    (9430387, HAND),
    (9430387, MZONE),
    (22912101, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, |duel, a| duel.has_flag_effect(a.tp, ID));
    e1.set_cost(duel, Cost::self_reveal);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_DESTROYED);
        ge1.set_operation(duel, descheckop);
        duel.register_effect(ge1, 0);
    });
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, negcon);
    e2.set_cost(duel, negcost);
    e2.set_target(duel, negtg);
    e2.set_operation(duel, negop);
    c.register_effect(duel, e2);
}

fn descheckop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    if re.get_handler(duel).is_set_card(duel, &[SET_BLITZCLIQUE]) {
        duel.register_flag_effect(a.rp, ID, RESET_PHASE | PHASE_END, 0, 1);
    }
    Ok(())
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, None, 1, PLAYER_EITHER, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    if ft > 0 {
        if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
            ft = 1;
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(
            tp,
            |c, duel| spfilter(c, duel, e, tp),
            tp,
            LOCATION_HAND,
            0,
            1,
            ft.min(3) as u8,
            None,
        )?;
        if !g.is_empty() {
            let ct = duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
            if ct > 0 {
                let fg = duel.get_field_group(tp, LOCATION_ONFIELD, LOCATION_ONFIELD);
                if !fg.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
                    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
                    let dg = fg.select(duel, tp, 1, ct as u8, None)?;
                    if !dg.is_empty() {
                        duel.hint_selection(dg);
                        duel.break_effect()?;
                        duel.destroy(dg, REASON_EFFECT)?;
                    }
                }
            }
        }
    }
    register_effect_monster_lock(duel, e.get_handler(duel), tp, aux::stringid(ID, 3));
    Ok(())
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED)
        && re.is_monster_effect(duel)
        && duel.is_chain_negatable(a.ev as u8)
        && duel.get_chain_info::<u32>(a.ev as u8, CHAININFO_TRIGGERING_LOCATION) != LOCATION_HAND
}

fn negcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_THUNDER) && c.is_faceup(duel) && c.is_able_to_hand_as_cost(duel)
}

fn negcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(negcostfilter, tp, LOCATION_MZONE, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_matching_card(tp, negcostfilter, tp, LOCATION_MZONE, 0, 1, 1, Some(c.into()))?;
    duel.hint_selection(g);
    duel.send_to_hand(g, PLAYER_NONE, REASON_COST)?;
    Ok(false)
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tp = a.tp;
    let Some(re) = a.re else { return Ok(false) };
    let rc = re.get_handler(duel);
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, tp, 0);
    if rc.is_destructable(duel) && rc.is_relate_to_effect(duel, re) {
        duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, tp, 0);
    }
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(re) = a.re
        && duel.negate_activation(a.ev as u8)
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}
