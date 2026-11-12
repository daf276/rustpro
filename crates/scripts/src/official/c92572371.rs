// Brotherhood of the Fire Fist - Buffalo
use crate::prelude::*;
use crate::testing::{GRAVE, HAND, MZONE, SZONE};

const ID: u32 = 92572371;

// In the Graveyard, with Brotherhood of the Fire Fist - Spirit in the hand and on the field and Fire Formation - Tenki on
// the field to send to the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, GRAVE),
    (ID, GRAVE),
    (1662004, HAND),
    (1662004, HAND),
    (1662004, MZONE),
    (57103969, SZONE),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, |duel, a, chk| {
        Cost::replaceable_full(duel, a, chk, spcost, |duel, _, a| duel.get_location_count(a.tp, LOCATION_MZONE) > 0)
    });
    e1.set_target(duel, sptg);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
}

fn spcostfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_location(duel, LOCATION_HAND) || c.is_faceup(duel))
        && c.is_set_card(duel, &[SET_FIRE_FIST, SET_FIRE_FORMATION])
        && c.is_able_to_grave_as_cost(duel)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(spcostfilter, tp, LOCATION_HAND | LOCATION_ONFIELD, 0, None);
    if chk == 0 {
        return Ok(aux::select_unselect_group_check(duel, g, 2, 2, &aux::chkf_mmz(tp, 1)));
    }
    let sg = aux::select_unselect_group(duel, g, 2, 2, &aux::chkf_mmz(tp, 1), tp, HINTMSG_TOGRAVE)?;
    duel.send_to_grave(sg, REASON_COST)?;
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
