// Ritual Raven
use crate::prelude::*;
use crate::testing::{Check, HAND};

struct Tribute;

impl WholeLevelTribute for Tribute {
    fn condition(c: CardId, duel: &Duel, _: EffectId) -> bool {
        c.is_attribute(duel, ATTRIBUTE_DARK)
    }
}

// The whole tribute for Magician of Black Chaos (DARK), Ritual Summoned by Contract with the Abyss.
pub const SUPPORT: &[(u32, u16)] = &[(69035382, HAND), (30208479, HAND)];
pub const CHECKS: &[Check] = &[Check::RitualMaterial];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Ritual::add_whole_level_tribute::<Tribute>(duel, c);
}
