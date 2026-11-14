// Trias Hierarchia
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 26866984;

// Copies in the hand and the Graveyard, with Mokey Mokeys (Fairies) on the field to Tribute.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (27288416, MZONE),
    (27288416, MZONE),
    (27288416, MZONE),
    (27288416, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_HAND | LOCATION_GRAVE);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_MAIN_END);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    e1.set_cost(duel, spcost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn spcheck(duel: &Duel, sg: CardSet, tp: u8, _: CardSet) -> (bool, bool) {
    (duel.get_mzone_count(tp, Some(sg)) > 0, false)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let fairy = |c: CardId, duel: &Duel| c.is_race(duel, RACE_FAIRY);
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, fairy, (1, 1), false, Some(&spcheck), None));
    }
    let g = duel.select_release_group_cost(tp, fairy, (1, 3), false, Some(&spcheck), None)?;
    let ct = duel.release(g, REASON_COST)?;
    e.set_label(duel, ct.into());
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    let ct = e.get_label(duel);
    if ct >= 2 {
        duel.set_possible_operation_info(0, CATEGORY_DESTROY, None, 1, 1 - tp, LOCATION_ONFIELD as i32);
    }
    if ct == 3 {
        duel.set_possible_operation_info(0, CATEGORY_DRAW, None, 1, tp, 2);
    }
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3300);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
        e1.set_value(duel, LOCATION_REMOVED as i32);
        c.register_effect_full(duel, e1, true);
        let ct = e.get_label(duel);
        if ct == 1 {
            return Ok(());
        }
        let g = duel.get_field_group(tp, 0, LOCATION_ONFIELD);
        if !g.is_empty() && ct >= 2 && duel.select_yes_no(tp, aux::stringid(ID, 1))? {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
            let sg = g.select(duel, tp, 1, 1, None)?;
            duel.hint_selection_full(sg, true);
            duel.break_effect()?;
            duel.destroy(sg, REASON_EFFECT)?;
        }
        if ct == 3 && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
            duel.break_effect()?;
            duel.draw(tp, 2, REASON_EFFECT)?;
        }
    }
    Ok(())
}
