// Genex Ally Birdman
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 64034255;

// Copies in hand, with Genex Controllers on the field to return to the hand.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (68505803, MZONE), (68505803, MZONE), (68505803, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_cost(duel, spcost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn cfilter(c: CardId, duel: &Duel, ft: i32) -> bool {
    c.is_faceup(duel) && c.is_able_to_hand_as_cost(duel) && (ft > 0 || c.get_sequence(duel) < 5)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    if chk == 0 {
        return Ok(ft > -1 && duel.is_existing_matching_card(|c, duel| cfilter(c, duel, ft), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_matching_card(tp, |c, duel| cfilter(c, duel, ft), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    let wind = g.get_first().is_some_and(|tc| tc.is_attribute(duel, ATTRIBUTE_WIND));
    e.set_label(duel, i64::from(wind));
    duel.send_to_hand(g, PLAYER_NONE, REASON_COST)?;
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    if duel.special_summon_step(c, 0, tp, tp, false, false, POS_FACEUP)? {
        if e.get_label(duel) == 1 {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_UPDATE_ATTACK);
            e1.set_value(duel, 500);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
            c.register_effect(duel, e1);
        }
        let e2 = duel.create_effect(c);
        e2.set_description(duel, 3300);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
        e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e2.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
        e2.set_value(duel, LOCATION_REMOVED as i32);
        c.register_effect_full(duel, e2, true);
    }
    duel.special_summon_complete()?;
    Ok(())
}
