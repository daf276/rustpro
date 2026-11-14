// Danger!? Tsuchinoko?
use crate::prelude::*;
use crate::testing::{Check, HAND};

const ID: u32 = 99745551;

pub const LISTED_NAMES: &[u32] = &[ID];
// Copies in the hand, to be discarded at random and to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (ID, HAND)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_HANDES + CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_cost(duel, spcost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).get_previous_location(duel) == LOCATION_HAND && a.r & REASON_DISCARD != 0
    });
    e2.set_target(duel, stock::special_summon_self_target_player_0);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_REMOVE);
    c.register_effect(duel, e3);
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(!c.is_public(duel) && c.get_flag_effect(duel, ID) == 0);
    }
    c.register_flag_effect(duel, ID, RESET_CHAIN, 0, 1);
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, a.tp, 1);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_field_group(tp, LOCATION_HAND, 0);
    if g.is_empty() {
        return Ok(());
    }
    duel.shuffle_hand(tp)?;
    duel.hint(HINT_SELECTMSG, 1 - tp, HINTMSG_DISCARD);
    let tc = g.random_select(duel, 1 - tp, 1);
    duel.break_effect()?;
    duel.send_to_grave(tc, REASON_EFFECT | REASON_DISCARD)?;
    if !duel.is_player_can_special_summon_any(tp) || duel.get_location_count(tp, LOCATION_MZONE) == 0 {
        return Ok(());
    }
    if !tc.get_first().is_some_and(|tc| tc.is_code(duel, ID)) {
        duel.break_effect()?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let filter = |c: CardId, duel: &Duel| c.is_code(duel, ID) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false);
        let sc = duel.select_matching_card(tp, filter, tp, LOCATION_HAND, 0, 1, 1, None)?;
        if !sc.is_empty() {
            // The Lua tests Duel.SpecialSummon's count for truth, and 0 is true in Lua.
            duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP)?;
            duel.draw(tp, 1, REASON_EFFECT)?;
        }
    }
    Ok(())
}
