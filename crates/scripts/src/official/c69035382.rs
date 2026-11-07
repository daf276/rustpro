// Contract with the Abyss
use crate::prelude::*;
use crate::testing::HAND;

struct Params;

impl RitualParams for Params {
    fn filter(c: CardId, duel: &Duel) -> bool {
        c.is_attribute(duel, ATTRIBUTE_DARK)
    }
}

// Ritual Summons Magician of Black Chaos (DARK), with Ritual Raven as the whole tribute.
pub const SUPPORT: &[(u32, u16)] = &[(30208479, HAND), (34334692, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Ritual::add_proc_equal::<Params>(duel, c);
}
