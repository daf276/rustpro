// Dark Calling
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND};

const ID: u32 = 12071500;

pub const LISTED_NAMES: &[u32] = &[CARD_DARK_FUSION];

struct Params;

impl FusionSummonParams for Params {
    const CHKF: Option<u64> = Some(FUSPROC_NOLIMIT);

    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        super::dark_calling(c.get_original_code(duel))
    }

    fn matfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_location(duel, LOCATION_HAND) && c.is_able_to_remove(duel)
    }

    fn extrafil(duel: &Duel, _: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        let filter = |c, duel: &Duel| Fusion::is_monster_filter(c, duel, CardId::is_able_to_remove);
        (
            Some(duel.get_matching_group(filter, tp, LOCATION_GRAVE | LOCATION_HAND, 0, None)),
            None,
        )
    }

    const EXTRAOP: Option<FusionMaterialFn> = Some(Fusion::banish_material);

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk == 0 {
            return Ok(true);
        }
        duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, a.tp, (LOCATION_HAND | LOCATION_GRAVE) as i32);
        Ok(false)
    }
}

// Evil HERO Dark Gaia in the Extra Deck, with Evil HERO Adusted Gold in the hand and Giant Soldier of Stone in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (58332301, EXTRA), (13650422, HAND), (13039848, GRAVE)];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(58332301)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Fusion::register_summon_eff::<Params>(duel, c);
}
