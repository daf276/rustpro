// Vanquish Soul Caesar Valius
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 91073013;

pub const LISTED_SERIES: &[u16] = &[SET_VANQUISH_SOUL];
// In the hand and on the field, with non-Dragon Vanquish Soul monsters on the field to return, and EARTH (Hollie Sue), DARK
// (Dr. Mad Love) and FIRE (Razen) monsters in the hand to reveal.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (29302858, MZONE),
    (29280200, MZONE),
    (93156774, HAND),
    (29280200, HAND),
    (29302858, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    e1.set_cost(duel, opccost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, opccost);
    e2.set_target(duel, vstg);
    e2.set_operation(duel, vsop);
    c.register_effect(duel, e2);
}

fn opccost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.get_flag_effect(a.tp, ID) == 0);
    }
    duel.register_flag_effect(a.tp, ID, RESET_CHAIN, 0, 1);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_VANQUISH_SOUL])
        && !c.is_race(duel, RACE_DRAGON)
        && c.is_faceup(duel)
        && c.is_able_to_hand(duel)
        && duel.get_mzone_count(tp, Some(c.into())) > 0
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && thfilter(chkc, duel, tp));
    }
    if chk == 0 {
        return Ok(
            duel.is_existing_target(|c, duel| thfilter(c, duel, tp), tp, LOCATION_MZONE, 0, 1, None)
                && c.is_can_be_special_summoned(duel, e, 0, tp, false, false),
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_target(tp, |c, duel| thfilter(c, duel, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e)
        && duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)? > 0
        && tc.is_location(duel, LOCATION_HAND)
        && c.is_relate_to_effect(duel, e)
    {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn vscostfilter(c: CardId, duel: &Duel, att: u32) -> bool {
    c.is_attribute(duel, att) && !c.is_public(duel)
}

fn vsrescon(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    let count = |att| sg.filter_count(duel, |c, duel| c.is_attribute(duel, att), None) > 0;
    (count(ATTRIBUTE_EARTH) && count(ATTRIBUTE_DARK) && count(ATTRIBUTE_FIRE), false)
}

fn vstg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let cg1 = duel.get_matching_group(|c, duel| vscostfilter(c, duel, ATTRIBUTE_EARTH), tp, LOCATION_HAND, 0, None);
    let b1 = !cg1.is_empty();
    let other = |c, duel: &Duel| vscostfilter(c, duel, ATTRIBUTE_DARK | ATTRIBUTE_FIRE);
    let cg2 = cg1.union(duel.get_matching_group(other, tp, LOCATION_HAND, 0, None));
    let dg = duel.get_matching_group(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, Some(e.get_handler(duel).into()));
    let b2 = !dg.is_empty() && aux::select_unselect_group_check(duel, cg2, 1, 3, &vsrescon);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    match op {
        Some(1) => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
            let g = cg1.select(duel, tp, 1, 1, None)?;
            duel.confirm_cards(1 - tp, g)?;
            duel.shuffle_hand(tp)?;
            e.set_category(duel, 0);
        }
        Some(2) => {
            let g = aux::select_unselect_group_full(duel, cg2, 1, 3, &vsrescon, tp, HINTMSG_CONFIRM, Some(&vsrescon), false)?;
            duel.confirm_cards(1 - tp, g)?;
            duel.shuffle_hand(tp)?;
            e.set_category(duel, CATEGORY_DESTROY);
            duel.set_operation_info(0, CATEGORY_DESTROY, Some(dg), 1, 0, 0);
        }
        _ => {}
    }
    Ok(false)
}

fn vsop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    match e.get_label(duel) {
        1 => {
            if c.is_facedown(duel) || !c.is_relate_to_effect(duel, e) {
                return Ok(());
            }
            let e1 = duel.create_effect(c);
            e1.set_description(duel, aux::stringid(ID, 4));
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_property(
                duel,
                EFFECT_FLAG_SINGLE_RANGE + EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT,
            );
            e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
            e1.set_range(duel, LOCATION_MZONE);
            e1.set_value(
                duel,
                Value::Effect(|duel, e, te| te.is_activated(duel) && te.get_owner_player(duel) != e.get_handler_player(duel)),
            );
            e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
            c.register_effect(duel, e1);
        }
        2 => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
            let g = duel.select_matching_card(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, Some(c.into()))?;
            if g.is_empty() {
                return Ok(());
            }
            duel.hint_selection_full(g, true);
            duel.destroy(g, REASON_EFFECT)?;
        }
        _ => {}
    }
    Ok(())
}
