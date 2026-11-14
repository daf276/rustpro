// Salamangreat Tiger
use crate::prelude::*;
use crate::testing::{HAND, MZONE};

const ID: u32 = 84755744;

pub const LISTED_SERIES: &[u16] = &[SET_SALAMANGREAT];
// In the hand and on the field, with Salamangreat Gazelle in the hand to discard.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, MZONE), (ID, MZONE), (26889158, HAND), (26889158, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, spcost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_NONTUNER);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, Value::Card(|duel, _, sc| i32::from(sc.is_attribute(duel, ATTRIBUTE_FIRE))));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_LVCHANGE);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_target(duel, lvtg);
    e3.set_operation(duel, lvop);
    c.register_effect(duel, e3);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SALAMANGREAT]) && c.is_discardable(duel)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(cfilter, tp, LOCATION_HAND, 0, 1, Some(c.into())));
    }
    duel.discard_hand_full(tp, cfilter, 1, 1, REASON_COST | REASON_DISCARD, Some(c.into()))?;
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn lvtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.has_level(duel));
    }
    duel.set_operation_info(0, CATEGORY_LVCHANGE, Some(c.into()), 1, a.tp, 0);
    Ok(false)
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, e) && c.has_level(duel) {
        let op = duel.select_effect(
            tp,
            &[(true, aux::stringid(ID, 2)), (c.is_level_above(duel, 2), aux::stringid(ID, 3))],
        )?;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_LEVEL);
        e1.set_value(duel, if op == Some(2) { -1 } else { 1 });
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
    }
    Ok(())
}
