// Linkerbell
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 54635100;
const HONEYBOT: u32 = 34472920;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);
}

// In the Extra Deck with Honeybots to give it three more Extra Deck cards than the opponent, Link Summoned with two Protrons.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (HONEYBOT, EXTRA),
    (HONEYBOT, EXTRA),
    (HONEYBOT, EXTRA),
    (HONEYBOT, EXTRA),
    (24154052, MZONE),
    (24154052, MZONE),
    (24154052, MZONE),
    (24154052, MZONE),
];
// No activated effect.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SPSUMMON_COST);
    e1.set_cost_condition(duel, spcost);
    c.register_effect(duel, e1);
}

fn spcost(duel: &Duel, _: EffectId, _: CardId, tp: u8, st: u32) -> bool {
    if st & SUMMON_TYPE_LINK != SUMMON_TYPE_LINK {
        return true;
    }
    duel.get_field_group_count(tp, LOCATION_EXTRA, 0) as i64 - duel.get_field_group_count(tp, 0, LOCATION_EXTRA) as i64 >= 3
}
