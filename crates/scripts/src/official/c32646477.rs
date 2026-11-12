// Dark Strike Fighter
use super::c37675907::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 32646477;

// In the Extra Deck, Synchro Summoned with Scrap Synchron (a Level 1 Tuner) and Summoned Skull (Level 6).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (16449363, MZONE),
    (16449363, MZONE),
    (70781052, MZONE),
    (70781052, MZONE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_category(duel, CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, _| duel.is_phase(PHASE_MAIN1));
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, stock::damage);
    c.register_effect(duel, e1);
}

fn costfilter(c: CardId, duel: &Duel) -> bool {
    c.get_level(duel) > 0
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, costfilter, (1, 1), false, None, None));
    }
    let g = duel.select_release_group_cost(tp, costfilter, (1, 1), false, None, None)?;
    if let Some(tc) = g.get_first() {
        a.e.set_label(duel, i64::from(tc.get_level(duel) * 200));
    }
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(true);
    }
    let label = e.get_label(duel);
    duel.set_target_player(1 - tp);
    duel.set_target_param(label as u32);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, label as i32);
    e.set_label(duel, 0);
    Ok(false)
}
