// Overload Fusion
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 3659803;

struct Params;

impl FusionSummonParams for Params {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_attribute(duel, ATTRIBUTE_DARK) && c.is_race(duel, RACE_MACHINE)
    }

    fn matfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_on_field(duel) && c.is_able_to_remove(duel)
    }

    fn extrafil(duel: &Duel, _: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        if duel.is_player_affected_by_effect(tp, CARD_SPIRIT_ELIMINATION).is_some() {
            return (None, None);
        }
        let filter = |c, duel: &Duel| Fusion::is_monster_filter(c, duel, CardId::is_able_to_remove);
        (Some(duel.get_matching_group(filter, tp, LOCATION_GRAVE, 0, None)), None)
    }

    const EXTRAOP: Option<FusionMaterialFn> = Some(Fusion::banish_material);

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk == 0 {
            return Ok(true);
        }
        duel.set_operation_info(0, CATEGORY_REMOVE, None, 0, a.tp, (LOCATION_MZONE | LOCATION_GRAVE) as i32);
        Ok(false)
    }
}

// Gatling Dragon in the Extra Deck, Fusion Summoned with Barrel Dragon and Blowback Dragon on the field.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (87751584, EXTRA), (81480460, MZONE), (25551951, MZONE)];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(87751584)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Fusion::register_summon_eff::<Params>(duel, c);
}
