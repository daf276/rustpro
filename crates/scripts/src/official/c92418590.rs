// Gimmick Puppet Dreary Doll
use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE};

const ID: u32 = 92418590;

pub const LISTED_SERIES: &[u16] = &[SET_GIMMICK_PUPPET];

// In the Graveyard with Scissor Arms there and on the field to banish.
pub const SUPPORT: &[(u32, u16)] = &[(ID, GRAVE), (29216967, GRAVE), (29216967, GRAVE), (29216967, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_code(duel, EFFECT_CANNOT_BE_XYZ_MATERIAL);
    e2.set_value(
        duel,
        Value::OptionalCard(|duel, _, c| i32::from(c.is_some_and(|c| !c.is_set_card(duel, &[SET_GIMMICK_PUPPET])))),
    );
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_GIMMICK_PUPPET])
        && c.is_able_to_remove_as_cost(duel)
        && c.is_monster(duel)
        && aux::sp_elim_filter(c, duel, true, false)
        && (duel.get_location_count(tp, LOCATION_MZONE) > 0 || (c.is_location(duel, LOCATION_MZONE) && c.get_sequence(duel) < 5))
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let handler = Some(a.e.get_handler(duel).into());
    let filter = |c, duel: &Duel| cfilter(c, duel, tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, handler));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, 1, handler)?;
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_can_be_special_summoned(duel, a.e, 0, a.tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}
