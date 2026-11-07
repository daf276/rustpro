// Dominus Spark
use crate::prelude::*;

use super::c40366667::restrict_attributes;

const ID: u32 = 6325660;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e0.set_code(duel, EFFECT_TRAP_ACT_IN_HAND);
    e0.set_condition(duel, |duel, a| {
        duel.get_custom_activity_count(ID, 1 - a.e.get_handler_player(duel), ACTIVITY_CHAIN) > 0
    });
    c.register_effect(duel, e0);
    duel.add_custom_activity_counter(ID, ACTIVITY_CHAIN, CounterFilter::Chain(chainfilter));
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
}

fn chainfilter(duel: &Duel, re: EffectId, _: u8, _: u8) -> bool {
    !(re.is_monster_effect(duel) && re.get_activate_location(duel) & (LOCATION_HAND | LOCATION_GRAVE) > 0)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && chkc.is_able_to_remove(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_able_to_remove, tp, 0, LOCATION_MZONE, 1, None));
    }
    let from_hand = e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && e.get_handler(duel).is_status(duel, STATUS_ACT_FROM_HAND);
    e.set_label(duel, from_hand.into());
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, CardId::is_able_to_remove, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, 1 - tp, LOCATION_HAND as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, opp: u8) -> bool {
    c.is_can_be_special_summoned(duel, e, 0, opp, false, false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let opp = 1 - tp;
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    let filter = |c, duel: &Duel| spfilter(c, duel, e, opp);
    if tc.is_relate_to_effect(duel, e)
        && duel.remove(tc, POS_FACEUP, REASON_EFFECT)? > 0
        && !duel.is_existing_matching_card(CardId::is_trap, tp, LOCATION_GRAVE, 0, 1, None)
        && duel.get_location_count_full(opp, LOCATION_MZONE, opp, LOCATION_REASON_TOFIELD, 0xff) > 0
        && duel.is_existing_matching_card(filter, opp, LOCATION_HAND, 0, 1, None)
        && duel.select_yes_no(opp, aux::stringid(ID, 1))?
    {
        duel.hint(HINT_SELECTMSG, opp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(opp, filter, opp, LOCATION_HAND, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.break_effect()?;
            duel.special_summon(g, 0, opp, opp, false, false, POS_FACEUP)?;
        }
    }
    restrict_attributes::<ID, { ATTRIBUTE_EARTH | ATTRIBUTE_WATER | ATTRIBUTE_FIRE | ATTRIBUTE_WIND }>(duel, e, tp);
    Ok(())
}
