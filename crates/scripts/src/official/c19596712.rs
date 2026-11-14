// Abyss-scale of Cetus
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};
use engine::constants::MSG_CHAIN_DISABLED;

const ID: u32 = 19596712;

pub const LISTED_SERIES: &[u16] = &[SET_MERMAIL];
// Mermail Abysspikes to equip, and the opponent's Reckless Greeds to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (58471134, MZONE),
    (58471134, MZONE),
    (37576645, SZONE | OPPONENT),
    (37576645, SZONE | OPPONENT),
];
// The negation is a continuous effect forming no chain link.
pub const CHECKS: &[Check] = &[Check::Resolves, Check::Message(MSG_CHAIN_DISABLED)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    super::c72932673::add_abyss_scale(duel, c, 800, negcon);
    aux::double_snare_validity(duel, c, LOCATION_SZONE);
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    let loc: u32 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_LOCATION);
    a.rp == 1 - a.tp && loc == LOCATION_SZONE && a.re.is_some_and(|re| re.is_trap_effect(duel)) && duel.is_chain_disablable(a.ev as u8)
}
