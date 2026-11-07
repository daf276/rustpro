// Code Talker
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

// Code Generator on the field and one in the hand as extra material.
pub const SUPPORT: &[(u32, u16)] = &[(53413628, EXTRA), (30114823, MZONE), (30114823, HAND)];
// No activated effect.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::Card(|duel, _, c| linked_monsters(duel, c) as i32 * 500));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e2.set_condition(duel, |duel, a| linked_monsters(duel, a.e.get_handler(duel)) > 0);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e3.set_value(duel, Value::EffectPlayer(|duel, e, _, tp| tp != e.get_handler_player(duel)));
    c.register_effect(duel, e3);
}

fn linked_monsters(duel: &Duel, c: CardId) -> usize {
    c.get_linked_group(duel).filter_count(duel, |c, duel| c.is_monster(duel), None)
}
