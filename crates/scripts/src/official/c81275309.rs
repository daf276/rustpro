// General Raiho of the Ice Barrier
use engine::constants::{MSG_CHAIN_DISABLED, MSG_SELECT_YESNO};

use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 81275309;
const LAST_CHAIN_ID: u64 = (ID as u64) << 8;

// On the field, with monsters of the opponent's to activate their effects (Snipe Hunter, from the hand and field).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (84290642, MZONE | OPPONENT),
    (84290642, MZONE | OPPONENT),
    (84290642, HAND | OPPONENT),
    (84290642, HAND | OPPONENT),
];
// A continuous effect forms no chain link, so the prompt to discard and the negation are checked.
pub const CHECKS: &[Check] = &[Check::Message(MSG_SELECT_YESNO), Check::Message(MSG_CHAIN_DISABLED)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_CHAIN_SOLVING);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_operation(duel, handes);
    c.register_effect(duel, e1);
}

fn handes(duel: &mut Duel, a: &Args) -> Script<()> {
    let (ev, tp) = (a.ev as u8, a.tp);
    let trig_loc: u32 = duel.get_chain_info(ev, CHAININFO_TRIGGERING_LOCATION);
    let chain_id: i64 = duel.get_chain_info(ev, CHAININFO_CHAIN_ID);
    if !(a.ep == 1 - tp
        && trig_loc == LOCATION_MZONE
        && chain_id != duel.get_script_state(LAST_CHAIN_ID)
        && a.re.is_some_and(|re| re.is_monster_effect(duel)))
    {
        return Ok(());
    }
    duel.set_script_state(LAST_CHAIN_ID, chain_id);
    if duel.get_field_group_count(tp, 0, LOCATION_HAND) > 0 && duel.select_yes_no(1 - tp, aux::stringid(ID, 0))? {
        duel.discard_hand(1 - tp, aux::TRUE, 1, 1, REASON_EFFECT | REASON_DISCARD)?;
        duel.break_effect()?;
    } else {
        duel.negate_effect(ev);
    }
    Ok(())
}
