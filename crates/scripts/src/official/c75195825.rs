// Master Pendulum, the Dracoslayer
use crate::prelude::*;
use crate::testing::PZONE;

const ID: u32 = 75195825;

// In the Pendulum Zone with a copy to target.
pub const SUPPORT: &[(u32, u16)] = &[(ID, PZONE), (ID, PZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_PZONE);
    e2.set_property(duel, EFFECT_FLAG_NO_TURN_RESET + EFFECT_FLAG_CARD_TARGET);
    e2.set_count_limit(duel, 1, 0);
    e2.set_target(duel, sctg);
    e2.set_operation(duel, scop);
    c.register_effect(duel, e2);
}

fn sctg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_PZONE));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, LOCATION_PZONE, LOCATION_PZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_PZONE, LOCATION_PZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

fn scop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    if !e.get_handler(duel).is_relate_to_effect(duel, e) {
        return Ok(());
    }
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
    {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
