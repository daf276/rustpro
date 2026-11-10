// Sprite's Blessing
use crate::prelude::*;
use crate::testing::HAND;

const ID: u32 = 37626500;

struct Params;

impl RitualParams for Params {
    fn filter(c: CardId, duel: &Duel) -> bool {
        c.is_attribute(duel, ATTRIBUTE_LIGHT)
    }
}

// Ritual Summons Cyber Angel Benten (LIGHT, Level 6) or Sauravis (Level 7), with Summoned Skull (Level 6) and Mokey Mokey (Level 1).
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (77235086, HAND), (4810828, HAND), (70781052, HAND), (27288416, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Ritual::add_proc_equal::<Params>(duel, c);
}
