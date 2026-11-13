// Honeybot
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 34472920;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_CYBERSE, Some(lc), sumtype, tp)
    }
}

// In the Extra Deck, Link Summoned with two Protrons (Cyberse Normal Monsters).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (24154052, MZONE),
    (24154052, MZONE),
    (24154052, MZONE),
    (24154052, MZONE),
];
// No activated effect.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1.set_target_card(duel, tgtg);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e2.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE + EFFECT_FLAG_SET_AVAILABLE);
    c.register_effect(duel, e2);
}

fn tgtg(duel: &Duel, e: EffectId, c: CardId) -> bool {
    e.get_handler(duel).get_linked_group(duel).is_contains(c)
}
