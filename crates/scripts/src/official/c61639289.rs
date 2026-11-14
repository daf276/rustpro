// Igknight Crusader
use crate::testing::{Check, DECK, PZONE};

const ID: u32 = 61639289;

pub const LISTED_SERIES: &[u16] = &[SET_IGKNIGHT];
// In the Pendulum Zone with another Igknight (Igknight Gallant), and Igknight Margrave (a FIRE Warrior) in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[(ID, PZONE), (96802306, PZONE), (50407691, DECK), (50407691, DECK)];

pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub use super::c67273917::initial_effect;
