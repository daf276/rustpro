// Lunalight Fusion
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 87931906;

const LUNALIGHT_PERFUME_DANCER: u32 = 81196066;

pub const LISTED_SERIES: &[u16] = &[SET_LUNALIGHT];

struct Params;

impl FusionSummonParams for Params {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_LUNALIGHT])
    }

    fn extrafil(duel: &Duel, e: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        let from_extra = |c: CardId, duel: &Duel| c.is_summon_location(duel, LOCATION_EXTRA);
        if duel.is_existing_matching_card(from_extra, tp, 0, LOCATION_MZONE, 1, None) {
            let eg = duel.get_matching_group(exfilter, tp, LOCATION_EXTRA | LOCATION_DECK, 0, None);
            if !eg.is_empty() {
                return (Some(eg), Some((fcheck, e.get_handler(duel))));
            }
        }
        (None, None)
    }

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk == 0 {
            return Ok(true);
        }
        duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 1, a.tp, (LOCATION_DECK | LOCATION_EXTRA) as i32);
        Ok(false)
    }
}

// In hand with Lunalight Silver Hound and Gold Leo in hand and on the field, and in the Deck for when the opponent controls
// a monster summoned from the Extra Deck, to fuse into Lunalight Perfume Dancer.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (LUNALIGHT_PERFUME_DANCER, EXTRA),
    (LUNALIGHT_PERFUME_DANCER, EXTRA),
    (35763582, HAND),
    (8379983, MZONE),
    (35763582, DECK),
    (8379983, DECK),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(LUNALIGHT_PERFUME_DANCER)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Fusion::create_summon_eff::<Params>(duel, c);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    c.register_effect(duel, e1);
}

fn fcheck(duel: &Duel, _: CardId, _: u8, sg: CardSet, _: CardId) -> bool {
    sg.filter_count(duel, |c, duel| c.is_location(duel, LOCATION_EXTRA | LOCATION_DECK), None) <= 1
}

fn exfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_set_card(duel, &[SET_LUNALIGHT]) && c.is_able_to_grave(duel)
}
