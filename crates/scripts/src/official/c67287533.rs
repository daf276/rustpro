// Spirit Message "N"
use crate::prelude::*;
use crate::testing::Check;

// Only restricts being Set and Special Summoned, so no effect resolves.
pub const CHECKS: &[Check] = &[Check::Report(&Check::Resolves)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    super::c31893528::initial_effect(duel, c);
}
