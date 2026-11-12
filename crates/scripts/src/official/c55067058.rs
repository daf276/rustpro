// Number 19: Freezadon
use crate::prelude::*;
use crate::testing::{Check, MZONE};

const ID: u32 = 55067058;
pub const XYZ_NUMBER: u32 = 19;

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(5);
    const CT: usize = 2;
}

// Number 94: Crystalzeros, whose detach cost it replaces; the replacement is no chain link, so it is only reported.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (62070231, MZONE), (62070231, MZONE)];
pub const CHECKS: &[Check] = &[Check::Report(&Check::Resolves)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_OVERLAY_REMOVE_REPLACE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, rcon);
    e1.set_returning_operation(duel, rop);
    c.register_effect(duel, e1);
}

fn rcon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    a.r & REASON_COST != 0
        && re.is_activated(duel)
        && re.is_active_type(duel, TYPE_XYZ)
        && a.e.get_handler(duel).check_remove_overlay_card(duel, a.tp, 1, REASON_EFFECT)
        && a.ep == a.e.get_owner_player(duel)
        && re.get_handler(duel).get_overlay_count(duel) as i64 >= i64::from(a.ev) - 1
}

fn rop(duel: &mut Duel, a: &Args) -> Script<u32> {
    Ok(a.e.get_handler(duel).remove_overlay_card(duel, a.tp, 1, 1, REASON_EFFECT)? as u32)
}
