// Giganticastle
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 63422098;

// In the Extra Deck, Synchro Summoned with Zeta (Level 4 Tuner) and Akihiron (Level 5).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (42026772, MZONE),
    (42026772, MZONE),
    (36904469, MZONE),
    (36904469, MZONE),
];
// Only continuous effects without a chain link: attacks show that it is on the field.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::Message(MSG_ATTACK)];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, Value::Card(val));
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_operation(duel, regop);
    c.register_effect(duel, e3);
}

fn val(duel: &Duel, e: EffectId, _: CardId) -> i32 {
    e.get_handler(duel).get_flag_effect_label(duel, ID).unwrap_or(0) as i32
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_synchro_summoned(duel) {
        let ct = c.get_material_count(duel) as i64 - 1;
        c.register_flag_effect_full(duel, ID, RESET_EVENT | RESETS_STANDARD_DISABLE, 0, 0, ct * 200, 0);
    }
    Ok(())
}
