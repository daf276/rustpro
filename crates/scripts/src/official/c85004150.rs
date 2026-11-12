// Digital Bug Rhinosebus
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 85004150;

// In the GY to revive with Xyz Reborn, since no Level 7 LIGHT Insect is ported for the Xyz Summon, with the opponent's Sangan to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, GRAVE),
    (ID, GRAVE),
    (26708437, HAND),
    (26708437, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(mfilter);
    const LV: Option<u32> = Some(7);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;
    const ALTERF: Option<AlterFilter> = Some(ovfilter);
    const DESC: u64 = aux::stringid(ID, 0);

    fn op_check(duel: &Duel, _: EffectId, tp: u8, mc: CardId) -> bool {
        mc.check_remove_overlay_card(duel, tp, 2, REASON_COST)
    }

    fn op(duel: &mut Duel, _: EffectId, tp: u8, mc: CardId) -> Script<bool> {
        mc.remove_overlay_card(duel, tp, 2, 2, REASON_COST)?;
        Ok(true)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_PIERCE);
    c.register_effect(duel, e3);
}

fn ovfilter(c: CardId, duel: &Duel, tp: u8, xyzc: CardId) -> bool {
    let rank = c.get_rank(duel);
    c.is_faceup(duel)
        && c.is_type_full(duel, TYPE_XYZ, Some(xyzc), SUMMON_TYPE_XYZ as u64, tp)
        && (rank == 5 || rank == 6)
        && c.is_race_full(duel, RACE_INSECT, Some(xyzc), SUMMON_TYPE_XYZ as u64, tp)
}

fn mfilter(c: CardId, duel: &Duel, xyz: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_race_full(duel, RACE_INSECT, Some(xyz), sumtype, tp) && c.is_attribute_full(duel, ATTRIBUTE_LIGHT, Some(xyz), sumtype, tp)
}

fn desfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_defense_above(duel, 0)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(desfilter, tp, 0, LOCATION_MZONE, 1, None));
    }
    let g = duel.get_matching_group(desfilter, tp, 0, LOCATION_MZONE, None);
    let dg = g.get_max_group(duel, |c, duel| c.get_defense(duel).into()).map(|(dg, _)| dg);
    duel.set_operation_info(0, CATEGORY_DESTROY, dg, dg.map_or(0, |dg| dg.len() as u8), 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(desfilter, a.tp, 0, LOCATION_MZONE, None);
    if let Some((dg, _)) = g.get_max_group(duel, |c, duel| c.get_defense(duel).into()) {
        duel.destroy(dg, REASON_EFFECT)?;
    }
    Ok(())
}
