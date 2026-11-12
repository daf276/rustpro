// Queen of Thorns
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};
use engine::constants::MSG_PAY_LPCOST;

const ID: u32 = 45500495;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp) && c.is_race_full(duel, RACE_PLANT, Some(sc), sumtype, tp)
    }
}

// On the field, with non-Plant monsters (Key Mace, Goblin Calligrapher) in the hand to Normal Summon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (1929294, HAND), (1929294, HAND), (12057781, HAND), (12057781, HAND)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_PAY_LPCOST)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SUMMON_COST);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_HAND, LOCATION_HAND);
    e1.set_target_card(duel, |duel, _, c| c.get_race(duel) != RACE_PLANT);
    e1.set_cost_condition(duel, |duel, _, _, tp, _| duel.check_lp_cost(tp, 1000));
    e1.set_operation(duel, acop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_SPSUMMON_COST);
    c.register_effect(duel, e2);
}

fn acop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.pay_lp_cost(a.tp, 1000)
}
