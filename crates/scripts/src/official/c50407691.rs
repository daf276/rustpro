// Igknight Margrave
use crate::testing::{Check, DECK, PZONE};

const ID: u32 = 50407691;

pub const LISTED_SERIES: &[u16] = &[SET_IGKNIGHT];
// In the Pendulum Zone with another Igknight (Igknight Paladin), and Igknight Squire (a FIRE Warrior) in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[(ID, PZONE), (24019092, PZONE), (24131534, DECK), (24131534, DECK)];

pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub use super::c67273917::initial_effect;
