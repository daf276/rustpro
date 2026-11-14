// Polymerization, an alias script (Duel.LoadCardScriptAlias) of 24094653
use crate::prelude::*;

pub const SUPPORT: &[(u32, u16)] = super::c24094653::SUPPORT;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    super::c24094653::initial_effect(duel, c);
}
