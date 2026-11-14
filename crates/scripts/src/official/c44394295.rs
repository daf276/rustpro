// Shaddoll Fusion
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND};

const ID: u32 = 44394295;

pub const LISTED_SERIES: &[u16] = &[SET_SHADDOLL];

struct Params;

impl FusionSummonParams for Params {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_SHADDOLL])
    }

    fn extrafil(duel: &Duel, _: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        if duel.is_existing_matching_card(|c, duel| c.is_summon_location(duel, LOCATION_EXTRA), tp, 0, LOCATION_MZONE, 1, None) {
            let filter = |c, duel: &Duel| Fusion::is_monster_filter(c, duel, CardId::is_able_to_grave);
            return (Some(duel.get_matching_group(filter, tp, LOCATION_DECK, 0, None)), None);
        }
        (None, None)
    }

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk != 0 {
            duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 0, a.tp, LOCATION_DECK as i32);
        }
        Ok(true)
    }
}

// El Shaddoll Meshahrail in the Extra Deck, Fusion Summoned from the hand with a Shaddoll, a DARK and an EARTH monster.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (32467459, EXTRA),
    (66675911, HAND),
    (66675911, HAND),
    (CARD_DARK_MAGICIAN, HAND),
    (66675911, DECK),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(32467459)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Fusion::create_summon_eff::<Params>(duel, c);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    c.register_effect(duel, e1);
}
