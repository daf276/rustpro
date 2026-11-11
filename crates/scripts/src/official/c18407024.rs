// Wattbetta
use super::c10209545::{hdcon, hdtg};
use crate::prelude::*;
use crate::testing::{Check, MZONE};

const ID: u32 = 18407024;

// Attacks directly into an empty field from several copies.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (ID, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_HANDES);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_BATTLE_DAMAGE);
    e1.set_condition(duel, hdcon);
    e1.set_target(duel, hdtg);
    e1.set_operation(duel, hdop);
    c.register_effect(duel, e1);
}

fn hdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_field_group(a.ep, LOCATION_HAND, 0);
    if g.is_empty() {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, 1 - tp, HINTMSG_DISCARD);
    let sg = g.select(duel, 1 - tp, 1, 1, None)?;
    duel.send_to_grave(sg, REASON_DISCARD | REASON_EFFECT)?;
    Ok(())
}
