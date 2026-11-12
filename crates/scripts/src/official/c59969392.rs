// Archfiend Zombie-Skull
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 59969392;

pub const LISTED_NAMES: &[u32] = &[33420078];
pub const MATERIAL: &[u32] = &[33420078];

// In the Extra Deck, Synchro Summoned with Plaguespreader Zombie (Level 2 Tuner) and two Level 2 Zombies.
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (ID, EXTRA), (33420078, MZONE), (52550973, MZONE), (61201220, MZONE)];

pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 2;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_summon_code(duel, Some(sc), sumtype, tp, &[33420078])
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, |c, duel| c.is_race_full(duel, RACE_ZOMBIE, Some(sc), sumtype, tp))
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_race(duel, RACE_ZOMBIE));
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
}
