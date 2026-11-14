// Helmer, Helmsman Fur Hire
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 67466547;

pub const LISTED_NAMES: &[u32] = &[ID];
pub const LISTED_SERIES: &[u16] = &[SET_FUR_HIRE];
// On the field with Fur Hire monsters in the hand to Special Summon and to discard.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, HAND), (47647354, HAND), (93850652, HAND), (31467949, HAND)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_count_limit(duel, 1, ID);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target(duel, super::c31467949::sptg::<ID>);
    e1.set_operation(duel, super::c31467949::spop::<ID>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, super::c93850652::thcon);
    e2.set_cost(duel, |duel, a, chk| Cost::replaceable(duel, a, chk, drcost));
    e2.set_target(duel, stock::draw_target::<1>);
    e2.set_operation(duel, stock::draw);
    c.register_effect(duel, e2);
}

fn drcfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_FUR_HIRE]) && c.is_discardable(duel)
}

fn drcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(drcfilter, a.tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(a.tp, drcfilter, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}
