// Igknight Gallant
use crate::testing::{Check, DECK, PZONE};

const ID: u32 = 96802306;

pub const LISTED_SERIES: &[u16] = &[SET_IGKNIGHT];
// In the Pendulum Zone with another Igknight (Igknight Margrave), and Igknight Paladin (a FIRE Warrior) in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[(ID, PZONE), (50407691, PZONE), (24019092, DECK), (24019092, DECK)];

pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub use super::c67273917::initial_effect;
