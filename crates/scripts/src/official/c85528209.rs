// Superheavy Samurai Beast Kyubi
use crate::prelude::*;
use crate::testing::{Check, MZONE};
use engine::constants::MSG_ATTACK;

const ID: u32 = 85528209;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp) && c.is_set_card_full(duel, &[SET_SUPERHEAVY_SAMURAI], Some(sc), sumtype, tp)
    }
}

// Two copies on the field to attack.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_DEFENSE_ATTACK);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, sccon);
    e2.set_value(duel, Value::Card(adval));
    c.register_effect(duel, e2);
}

fn sccon(duel: &Duel, a: &Args) -> bool {
    !duel.is_existing_matching_card(CardId::is_spell_trap, a.e.get_handler_player(duel), LOCATION_GRAVE, 0, 1, None)
}

fn adval(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    duel.get_matching_group_count(ctfilter, c.get_controler(duel), 0, LOCATION_MZONE, None) as i32 * 900
}

fn ctfilter(c: CardId, duel: &Duel) -> bool {
    c.get_summon_type(duel) & SUMMON_TYPE_SPECIAL == SUMMON_TYPE_SPECIAL
}
