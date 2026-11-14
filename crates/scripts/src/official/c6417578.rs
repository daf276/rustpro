// El Shaddoll Fusion
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};

const ID: u32 = 6417578;

pub const LISTED_SERIES: &[u16] = &[SET_SHADDOLL];

struct Params;

impl FusionSummonParams for Params {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_SHADDOLL])
    }
}

// El Shaddoll Winda in the Extra Deck, Fusion Summoned with Shaddoll Squamata and Shaddoll Dragon from the hand.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (94977269, EXTRA), (30328508, HAND), (77723643, HAND)];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(94977269)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Fusion::create_summon_eff::<Params>(duel, c);
    e1.set_hint_timing(duel, 0, TIMING_END_PHASE);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    c.register_effect(duel, e1);
}
