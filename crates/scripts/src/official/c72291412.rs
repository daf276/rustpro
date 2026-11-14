// D/D Necro Slime
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE};

const ID: u32 = 72291412;

pub const LISTED_SERIES: &[u16] = &[SET_DDD];

struct Params;

impl FusionSummonParams for Params {
    const EXTRAOP: Option<FusionMaterialFn> = Some(Fusion::banish_material);

    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_DDD])
    }

    fn matfilter(_: CardId, _: &Duel, _: EffectId, _: u8) -> bool {
        false
    }

    fn extrafil(duel: &Duel, _: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        if duel.is_player_affected_by_effect(tp, CARD_SPIRIT_ELIMINATION).is_none() {
            let filter = |c, duel: &Duel| Fusion::is_monster_filter(c, duel, CardId::is_able_to_remove);
            return (Some(duel.get_matching_group(filter, tp, LOCATION_GRAVE, 0, None)), None);
        }
        (None, None)
    }

    fn gc(duel: &Duel, e: EffectId, _: u8) -> Option<CardSet> {
        Some(Fusion::forced_handler(duel, e))
    }

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk == 0 {
            return Ok(true);
        }
        let c = a.e.get_handler(duel);
        duel.set_operation_info(0, CATEGORY_REMOVE, Some(c.into()), 0, a.tp, LOCATION_GRAVE as i32);
        Ok(false)
    }
}

// In the Graveyard with D/D monsters there, and D/D/D Fusion Monsters in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, GRAVE),
    (ID, GRAVE),
    (72181263, GRAVE),
    (33334269, GRAVE),
    (74583607, EXTRA),
    (11852093, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(74583607)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, Fusion::summon_eff_tg::<Params>);
    e1.set_operation(duel, Fusion::summon_eff_op::<Params>);
    c.register_effect(duel, e1);
}
