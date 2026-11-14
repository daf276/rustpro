// Ritual Beast Tamer Elder
use crate::prelude::*;
use crate::testing::{Check, HAND};

const ID: u32 = 14513016;

pub const LISTED_SERIES: &[u16] = &[SET_RITUAL_BEAST];
// In the hand to Normal Summon, with copies and Ritual Beast monsters to use the extra Normal Summon on.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (88123329, HAND), (88123329, HAND)];
// The only effect is continuous and forms no chain link: its Normal Summon is what grants the extra one.
pub const CHECKS: &[Check] = &[Check::TributeSummoned(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.set_sp_summon_once(duel, ID);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_operation(duel, sumop);
    c.register_effect(duel, e1);
}

fn sumop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if duel.get_flag_effect(tp, ID) != 0 {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_target_range(duel, LOCATION_HAND | LOCATION_MZONE, 0);
    e1.set_code(duel, EFFECT_EXTRA_SUMMON_COUNT);
    e1.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_RITUAL_BEAST]));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
    Ok(())
}
