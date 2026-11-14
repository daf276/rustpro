// Darklord Dance
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 99941223;

pub const LISTED_SERIES: &[u16] = &[SET_DARKLORD];

const DARKLORD_EVENINGSTAR: u32 = 10136446;
const DARKLORD_GULGOLET: u32 = 84031359;
const DARKLORD_IXCHEL: u32 = 52840267;

struct Params;

impl FusionSummonParams for Params {
    const EXTRAOP: Option<FusionMaterialFn> = Some(Fusion::banish_material);

    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_attribute(duel, ATTRIBUTE_DARK) && c.is_race(duel, RACE_FAIRY)
    }

    // Card.IsAbleToRemove(c, e, tp, chk): the effect is no player, so tp is the position and chk the reason.
    fn matfilter_chk(c: CardId, duel: &Duel, _: EffectId, tp: u8, chk: u32) -> bool {
        c.is_able_to_remove_full(duel, None, u32::from(tp), chk)
    }

    fn extrafil(duel: &Duel, e: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        let check = Some((fcheck as fn(&Duel, CardId, u8, CardSet, CardId) -> bool, e.get_handler(duel)));
        if duel.is_player_affected_by_effect(tp, CARD_SPIRIT_ELIMINATION).is_some() {
            return (None, check);
        }
        let filter = |c, duel: &Duel| fextrafilter(c, duel, tp);
        (
            Some(duel.get_matching_group(filter, tp, LOCATION_GRAVE, LOCATION_MZONE, None)),
            check,
        )
    }

    fn stage2(duel: &mut Duel, e: EffectId, tc: Option<CardId>, _: u8, _: Option<CardSet>, chk: i32) -> Script<()> {
        let Some(fc) = tc.filter(|_| chk == 0) else { return Ok(()) };
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, 1000);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        fc.register_effect(duel, e1);
        Ok(())
    }

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk == 0 {
            return Ok(true);
        }
        duel.set_operation_info(
            0,
            CATEGORY_REMOVE,
            None,
            1,
            a.tp,
            (LOCATION_HAND | LOCATION_MZONE | LOCATION_GRAVE) as i32,
        );
        Ok(false)
    }
}

// In the hand, to Fusion Summon Darklord Eveningstar from Darklord Gulgolet and Ixchel.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (DARKLORD_EVENINGSTAR, EXTRA),
    (DARKLORD_GULGOLET, HAND),
    (DARKLORD_GULGOLET, MZONE),
    (DARKLORD_IXCHEL, GRAVE),
    (DARKLORD_IXCHEL, HAND),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(DARKLORD_EVENINGSTAR)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Fusion::create_summon_eff::<Params>(duel, c);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    c.register_effect(duel, e1);
}

fn fcheck(duel: &Duel, _: CardId, tp: u8, sg: CardSet, fc: CardId) -> bool {
    fc.is_set_card(duel, &[SET_DARKLORD]) || !sg.is_exists(duel, |c, duel| c.is_controler(duel, 1 - tp), 1, None)
}

fn fextrafilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_monster(duel)
        && c.is_able_to_remove(duel)
        && (c.is_controler(duel, tp) || (c.is_set_card(duel, &[SET_DARKLORD]) && c.is_faceup(duel)))
}
