// Number 10: Illumiknight
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE};

const ID: u32 = 11411223;
pub const XYZ_NUMBER: u32 = 10;

// In the Extra Deck, Xyz Summoned with three Level 4 monsters.
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (2118022, MZONE), (2483611, MZONE), (4042268, MZONE), (5628232, MZONE)];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 3;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_field_group_count(tp, LOCATION_HAND, 0) > 0 && duel.is_player_can_draw(tp, 1));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_HAND as i32);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, aux::TRUE, tp, LOCATION_HAND, 0, 1, 1, None)?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    duel.send_to_grave(g, REASON_EFFECT)?;
    if tc.is_location(duel, LOCATION_GRAVE) {
        duel.draw(tp, 1, REASON_EFFECT)?;
    }
    Ok(())
}
