// Metalfoes Volflame
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE, PZONE};

const ID: u32 = 69351984;

// In the Pendulum Zone, with a monster on the field to destroy and Metalfoes Fusion (a Metalfoes Spell) in the Deck to Set.
pub const SUPPORT: &[(u32, u16)] = &[(ID, PZONE), (ID, MZONE), (73594093, DECK), (73594093, DECK), (73594093, DECK)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    super::c7868571::initial_effect(duel, c);
}
