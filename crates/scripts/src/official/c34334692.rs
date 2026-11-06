// Ritual Raven
use crate::prelude::*;

struct Tribute;

impl WholeLevelTribute for Tribute {
    fn condition(c: CardId, duel: &Duel, _: EffectId) -> bool {
        c.is_attribute(duel, ATTRIBUTE_DARK)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Ritual::add_whole_level_tribute::<Tribute>(duel, c);
}
