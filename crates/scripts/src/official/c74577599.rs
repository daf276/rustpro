// Traptrix Holeutea
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, SZONE};

const ID: u32 = 74577599;

pub const LISTED_SERIES: &[u16] = &[SET_TRAPTRIX];
// Set copies and one in the GY, Trap Hole in the hand to discard, and Traptrix Myrmeleo in the GY to revive.
pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE), (ID, SZONE), (ID, GRAVE), (4206964, HAND), (91812341, GRAVE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::ResolvesEffect(aux::stringid(ID, 2))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, selfspcost);
    e1.set_target(duel, selfsptg);
    e1.set_operation(duel, selfspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e2.set_code(duel, EFFECT_TRAP_ACT_IN_SET_TURN);
    e2.set_value(duel, Value::Activation(|duel, e, _| e.set_label(duel, 1)));
    e2.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(selfspcostfilter, a.e.get_handler_player(duel), LOCATION_HAND, 0, 1, None)
    });
    c.register_effect(duel, e2);
    e1.set_label_object(duel, Some(e2));
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e3.set_count_limit(duel, 1, ID);
    e3.set_cost(duel, Cost::self_banish);
    e3.set_target(duel, gysptg);
    e3.set_operation(duel, stock::special_summon_first_target::<POS_FACEUP>);
    c.register_effect(duel, e3);
}

fn selfspcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_normal_trap(duel) && c.is_discardable(duel)
}

fn selfspcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let Some(label_obj) = a.e.get_label_object(duel) else {
        return Ok(chk == 0);
    };
    if chk == 0 {
        label_obj.set_label(duel, 0);
        return Ok(true);
    }
    if label_obj.get_label(duel) > 0 {
        label_obj.set_label(duel, 0);
        duel.discard_hand(a.tp, selfspcostfilter, 1, 1, REASON_COST | REASON_DISCARD)?;
    }
    Ok(false)
}

fn can_summon(duel: &mut Duel, tp: u8) -> bool {
    let stats = MonsterStats::new(SET_TRAPTRIX, TYPE_MONSTER | TYPE_NORMAL, 400, 2400, 4, RACE_PLANT, ATTRIBUTE_EARTH);
    duel.is_player_can_special_summon_monster_full(tp, ID, stats, POS_FACEUP_DEFENSE, tp, 0)
}

fn selfsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && can_summon(duel, tp));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(e.get_handler(duel).into()), 1, tp, 0);
    Ok(false)
}

fn selfspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if c.is_relate_to_effect(duel, a.e) && can_summon(duel, tp) {
        c.add_monster_attribute(duel, TYPE_NORMAL);
        duel.special_summon_step(c, 0, tp, tp, true, false, POS_FACEUP_DEFENSE)?;
        c.add_monster_attribute_complete(duel);
        duel.special_summon_complete()?;
    }
    Ok(())
}

fn gyspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_TRAPTRIX]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn gysptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let filter = |c, duel: &Duel| gyspfilter(c, duel, e, tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_target(filter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, filter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    Ok(false)
}
