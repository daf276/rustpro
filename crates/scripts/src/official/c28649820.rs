// Embodiment of Apophis
use crate::prelude::*;
use crate::testing::{Check, HAND, SZONE};

const ID: u32 = 28649820;

pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, SZONE)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn can_summon(duel: &mut Duel, tp: u8) -> bool {
    let stats = MonsterStats::new(0, TYPE_MONSTER | TYPE_NORMAL, 1600, 1800, 4, RACE_REPTILE, ATTRIBUTE_EARTH);
    duel.is_player_can_special_summon_monster(tp, ID, stats)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && can_summon(duel, tp));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(a.e.get_handler(duel).into()), 1, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 || !can_summon(duel, tp) {
        return Ok(());
    }
    c.add_monster_attribute(duel, TYPE_NORMAL + TYPE_TRAP);
    duel.special_summon_step(c, 0, tp, tp, true, false, POS_FACEUP)?;
    c.add_monster_attribute_complete(duel);
    duel.special_summon_complete()?;
    Ok(())
}
