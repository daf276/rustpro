// Polymerization
use crate::prelude::*;

struct Params;

impl FusionSummonParams for Params {}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Fusion::register_summon_eff::<Params>(duel, c);
}
