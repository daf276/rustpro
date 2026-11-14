// Igknight Squire
use crate::prelude::SET_IGKNIGHT;
use crate::testing::{Check, DECK, PZONE};

const ID: u32 = 24131534;

pub const LISTED_SERIES: &[u16] = &[SET_IGKNIGHT];
// In the Pendulum Zone with another Igknight (Igknight Templar), and Igknight Cavalier (a FIRE Warrior) in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[(ID, PZONE), (97024987, PZONE), (67273917, DECK), (67273917, DECK)];

pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub use super::c67273917::initial_effect;
