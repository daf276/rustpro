// Rahu Dracotail
use super::c7375867::extra_deck_limit;
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND};

const ID: u32 = 32548318;

pub const LISTED_SERIES: &[u16] = &[SET_DRACOTAIL];

struct Params;

impl FusionSummonParams for Params {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_DRACOTAIL])
    }

    fn extrafil(duel: &Duel, _: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        let filter = |c, duel: &Duel| Fusion::is_monster_filter(c, duel, CardId::is_able_to_grave);
        (Some(duel.get_matching_group(filter, tp, LOCATION_DECK, 0, None)), None)
    }

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk != 0 {
            duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 0, a.tp, LOCATION_DECK as i32);
        }
        Ok(true)
    }

    fn stage2(duel: &mut Duel, e: EffectId, _: Option<CardId>, tp: u8, _: Option<CardSet>, chk: i32) -> Script<()> {
        if chk == 2 && e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
            extra_deck_limit(duel, e.get_handler(duel), tp, aux::stringid(ID, 1));
        }
        Ok(())
    }
}

// Dracotail Arthalion in the Extra Deck, Fusion Summoned with Dracotail monsters in hand and the Deck.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (33760966, EXTRA), (75003700, HAND), (75003700, DECK), (1498449, DECK)];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(33760966)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Fusion::create_summon_eff::<Params>(duel, c);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    c.register_effect(duel, e1);
}
