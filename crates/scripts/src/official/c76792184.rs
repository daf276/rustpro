// Black Magic Ritual
use crate::prelude::*;

pub const FIT_MONSTER: &[u32] = &[30208479];

struct Params;

impl RitualCode for Params {
    const LV: Option<u32> = Some(8);
    const CODES: &'static [u32] = FIT_MONSTER;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Ritual::add_proc_greater_code::<Params>(duel, c);
}
