// Possessed Dark Soul
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 52860176;

// On the field to take Sangan of the opponent's.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (26202165, MZONE | OPPONENT), (26202165, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_CONTROL);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, Cost::self_tribute);
    e1.set_target(duel, controltg);
    e1.set_operation(duel, controlop);
    c.register_effect(duel, e1);
}

fn controlfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level_below(duel, 3) && c.is_faceup(duel) && c.is_controler_can_be_changed_full(duel, true, 0xff)
}

fn controltg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let ctrl_count = duel.get_matching_group_count(controlfilter, tp, 0, LOCATION_MZONE, None) as i32;
        let handler = a.e.get_handler(duel);
        return Ok(ctrl_count > 0 && duel.get_mzone_count_full(tp, Some(handler.into()), tp, LOCATION_REASON_CONTROL, 0xff) >= ctrl_count);
    }
    duel.set_operation_info(0, CATEGORY_CONTROL, None, 1, 1 - tp, LOCATION_MZONE as i32);
    Ok(false)
}

fn controlop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(controlfilter, tp, 0, LOCATION_MZONE, None);
    if !g.is_empty() {
        duel.get_control(g, tp)?;
    }
    Ok(())
}
