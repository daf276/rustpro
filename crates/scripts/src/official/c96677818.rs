// Spellbook Organization
use crate::prelude::*;
use crate::testing::HAND;

const ID: u32 = 96677818;

pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, |duel, a, chk, _| {
        Ok(chk == 0 && duel.get_field_group_count(a.tp, LOCATION_DECK, 0) > 2)
    });
    e1.set_operation(duel, |duel, a| duel.sort_decktop(a.tp, a.tp, 3));
    c.register_effect(duel, e1);
}
