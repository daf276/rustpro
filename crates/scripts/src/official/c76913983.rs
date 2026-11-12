// Blackwing Armed Wing
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};
use engine::constants::MSG_ATTACK;

const ID: u32 = 76913983;

// Blackwing Tuner + non-Tuners; Full Armor Master and Silverwind reuse it with their own minimum of non-Tuners.
pub struct Procedure<const MIN: usize>;

impl<const MIN: usize> SynchroProcedure for Procedure<MIN> {
    const MIN2: usize = MIN;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_BLACKWING], Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// On the field and in the Extra Deck (Synchro Summoned from Blizzard, a Level 2 Tuner, and Zephyros, Level 4), attacking the
// opponent's Blue-Eyes White Dragons.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, EXTRA),
    (22835145, MZONE),
    (22835145, MZONE),
    (14785765, MZONE),
    (14785765, MZONE),
    (89631139, MZONE | OPPONENT),
    (89631139, MZONE | OPPONENT),
];
// Only continuous effects.
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure<1>>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_condition(duel, atkcon);
    e1.set_value(duel, 500);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_PIERCE);
    c.register_effect(duel, e2);
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    let ph = duel.get_current_phase();
    if ph != PHASE_DAMAGE && ph != PHASE_DAMAGE_CAL {
        return false;
    }
    duel.get_attacker() == Some(a.e.get_handler(duel)) && duel.get_attack_target().is_some_and(|d| d.is_defense_pos(duel))
}
