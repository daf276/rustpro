// Polymerization
use crate::prelude::*;
use crate::testing::HAND;

struct Params;

impl FusionSummonParams for Params {}

// Fallen of Albaz and a LIGHT monster, Petit Angel, for Albion the Branded Dragon in the Elfnote Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[(68468459, HAND), (38142739, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Fusion::register_summon_eff::<Params>(duel, c);
}
