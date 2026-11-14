// Legendary Six Samurai - Mizuho
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 74094021;
const SHINAI: u32 = 48505422;

pub const LISTED_SERIES: &[u16] = &[SET_SIX_SAMURAI];
pub const LISTED_NAMES: &[u32] = &[SHINAI];
// In the hand with Shinai on the field to summon it, and Six Samurai (Kageki) to Tribute.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, MZONE), (SHINAI, MZONE), (SHINAI, MZONE), (2511717, MZONE)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, spcon::<SHINAI>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_count_limit(duel, 1, 0);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_cost(duel, cost);
    e2.set_target(duel, target);
    e2.set_operation(duel, stock::destroy_first_target);
    c.register_effect(duel, e2);
}

// Shared with Shinai, which needs Mizuho face-up instead.
pub fn spcon<const CODE: u32>(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(
            |c, duel| c.is_faceup(duel) && c.is_code(duel, CODE),
            tp,
            LOCATION_ONFIELD,
            0,
            1,
            None,
        )
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SIX_SAMURAI])
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let dg = duel.get_matching_group(
        |c, duel| c.is_can_be_effect_target(duel, e),
        tp,
        LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        None,
    );
    let check = aux::release_check_target(dg);
    let handler = Some(e.get_handler(duel).into());
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, cfilter, (1, 1), false, Some(&check), handler));
    }
    let sg = duel.select_release_group_cost(tp, cfilter, (1, 1), false, Some(&check), handler)?;
    duel.release(sg, REASON_COST)?;
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}
