// Magical Musketeer Doc
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 68246154;

pub const LISTED_SERIES: &[u16] = &[SET_MAGICAL_MUSKET];

const DESPERADO: u32 = 20745268;

// On the field in several columns, with Magical Musket - Desperado in the hand to activate in one of them, and Magical Musketeer
// Kidbrave in the GY to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (DESPERADO, HAND),
    (DESPERADO, HAND),
    (DESPERADO, HAND),
    (DESPERADO, HAND),
    (5230799, GRAVE),
    (5230799, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    super::c32841045::register::<ID, LOCATION_GRAVE>(duel, c);
}
