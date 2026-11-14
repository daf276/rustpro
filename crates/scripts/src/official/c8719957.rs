// Abyss-scale of the Kraken
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};
use engine::constants::MSG_CHAIN_DISABLED;

const ID: u32 = 8719957;

pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (58471134, MZONE), (58471134, MZONE)];
// The negation is a continuous effect forming no chain link.
pub const CHECKS: &[Check] = &[Check::Resolves, Check::Message(MSG_CHAIN_DISABLED)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    super::c72932673::add_abyss_scale(duel, c, 400, negcon);
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    let loc: u32 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_LOCATION);
    a.rp != a.tp && loc == LOCATION_MZONE && a.re.is_some_and(|re| re.is_monster_effect(duel)) && duel.is_chain_disablable(a.ev as u8)
}
