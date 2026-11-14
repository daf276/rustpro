// Double Interlock
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND};

const ID: u32 = 38129297;

pub const LISTED_SERIES: &[u16] = &[SET_ROKKET];
// Fighting Flame Dragon is Fusion Summoned and Borreload Riot Dragon Ritual Summoned, by banishing Rokket Loader and Rokket Caliber
// (Dragons) and Burstinatrix (FIRE Warrior) from the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (36319131, EXTRA),
    (7987191, HAND),
    (25554552, GRAVE),
    (25554552, GRAVE),
    (67127799, GRAVE),
    (58932615, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(36319131), Check::SpecialSummonedCode(7987191)];

struct FusionEffect;

impl FusionSummonParams for FusionEffect {
    fn matfilter(_: CardId, _: &Duel, _: EffectId, _: u8) -> bool {
        false
    }

    fn extrafil(duel: &Duel, e: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        let check: Option<FusionCheck> = Some((matcheck, e.get_handler(duel)));
        if duel.is_player_affected_by_effect(tp, CARD_SPIRIT_ELIMINATION).is_some() {
            return (None, check);
        }
        let filter = |c, duel: &Duel| Fusion::is_monster_filter(c, duel, CardId::is_able_to_remove);
        (Some(duel.get_matching_group(filter, tp, LOCATION_GRAVE, 0, None)), check)
    }

    const EXTRAOP: Option<FusionMaterialFn> = Some(Fusion::banish_material);

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        extratg(duel, a, chk)
    }
}

struct RitualEffect;

impl RitualParams for RitualEffect {
    const FORCEDSELECTION: Option<ForcedSelection> =
        Some(|duel, _, _, sg, _| (sg.is_exists(duel, |c, duel| c.is_set_card(duel, &[SET_ROKKET]), 1, None), false));

    fn matfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_location(duel, LOCATION_GRAVE)
    }

    fn extrafil(duel: &Duel, _: EffectId, tp: u8) -> Option<CardSet> {
        if duel.is_player_affected_by_effect(tp, CARD_SPIRIT_ELIMINATION).is_some() {
            return None;
        }
        let filter = |c: CardId, duel: &Duel| c.has_level(duel) && c.is_able_to_remove(duel);
        Some(duel.get_matching_group(filter, tp, LOCATION_GRAVE, 0, None))
    }

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        extratg(duel, a, chk)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_cost(duel, effcost);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
}

fn matcheck(duel: &Duel, _: CardId, tp: u8, sg: CardSet, fc: CardId) -> bool {
    sg.is_exists(
        duel,
        |c, duel| c.is_set_card_full(duel, &[SET_ROKKET], Some(fc), SUMMON_TYPE_FUSION as u64, tp),
        1,
        None,
    )
}

fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, a.tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn fusion_target(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    Fusion::new_summon_eff_closure(duel, a.e, false);
    Fusion::summon_eff_tg::<FusionEffect>(duel, a, chk, None)
}

fn effcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    a.e.set_label(duel, -100);
    let b1 = duel.get_flag_effect(tp, ID) == 0 && fusion_target(duel, a, 0)?;
    let b2 = duel.get_flag_effect(tp, ID + 1) == 0 && Ritual::target::<RitualEffect, RITPROC_GREATER>(duel, a, 0, None)?;
    Ok(chk == 0 && (b1 || b2))
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let cost_skip = e.get_label(duel) != -100;
    let b1 = (cost_skip || duel.get_flag_effect(tp, ID) == 0) && fusion_target(duel, a, 0)?;
    let b2 = (cost_skip || duel.get_flag_effect(tp, ID + 1) == 0) && Ritual::target::<RitualEffect, RITPROC_GREATER>(duel, a, 0, None)?;
    if chk == 0 {
        e.set_label(duel, 0);
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    match op {
        Some(1) => {
            e.set_category(duel, CATEGORY_REMOVE + CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
            if !cost_skip {
                duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
            }
            fusion_target(duel, a, 1)?;
        }
        Some(2) => {
            e.set_category(duel, CATEGORY_REMOVE + CATEGORY_SPECIAL_SUMMON);
            if !cost_skip {
                duel.register_flag_effect(tp, ID + 1, RESET_PHASE | PHASE_END, 0, 1);
            }
            Ritual::target::<RitualEffect, RITPROC_GREATER>(duel, a, 1, None)?;
        }
        _ => {}
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    match a.e.get_label(duel) {
        1 => {
            Fusion::new_summon_eff_closure(duel, a.e, true);
            Fusion::summon_eff_op::<FusionEffect>(duel, a)
        }
        2 => Ritual::operation::<RitualEffect, RITPROC_GREATER>(duel, a),
        _ => Ok(()),
    }
}
