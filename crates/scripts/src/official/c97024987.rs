// Igknight Templar
use crate::prelude::SET_IGKNIGHT;
use crate::testing::{Check, DECK, PZONE};

const ID: u32 = 97024987;

pub const LISTED_SERIES: &[u16] = &[SET_IGKNIGHT];
// In the Pendulum Zone with another Igknight (Igknight Cavalier), and Igknight Crusader (a FIRE Warrior) in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[(ID, PZONE), (67273917, PZONE), (61639289, DECK), (61639289, DECK)];

pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub use super::c67273917::initial_effect;
