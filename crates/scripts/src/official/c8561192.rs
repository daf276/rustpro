// Leo, the Keeper of the Sacred Tree
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 8561192;

// In the Extra Deck, Synchro Summoned with Labradorite Dragon (Level 6 Tuner) and Gokibore (Level 4).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (62514770, MZONE),
    (62514770, MZONE),
    (15367030, MZONE),
    (15367030, MZONE),
];
// A continuous effect with no operation: attacks show that it is on the field.
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
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e1.set_condition(duel, tgcon);
    e1.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e1);
}

fn tgcon(duel: &Duel, a: &Args) -> bool {
    duel.get_turn_player() != a.e.get_handler_player(duel) || duel.get_current_phase() != PHASE_MAIN2
}
