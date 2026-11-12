// Star Eater
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 41517789;

// In the Extra Deck, Synchro Summoned with Labradorite Dragon (Level 6 Tuner) and Akihiron (Level 5).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (62514770, MZONE),
    (62514770, MZONE),
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
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_value(duel, Value::SpSummonCondition(aux::synlimit));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_CANNOT_DISABLE_SPSUMMON);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_operation(duel, sumsuc);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e4.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e4.set_operation(duel, atkop);
    c.register_effect(duel, e4);
}

fn sumsuc(duel: &mut Duel, a: &Args) -> Script<()> {
    if !a.e.get_handler(duel).is_synchro_summoned(duel) {
        return Ok(());
    }
    duel.set_chain_limit_till_chain_end(|_, _, _, _, _| false, 0);
    Ok(())
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e1.set_value(duel, Value::Effect(efilter));
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_DAMAGE);
    c.register_effect(duel, e1);
    duel.adjust_instantly(Some(c));
    Ok(())
}

fn efilter(duel: &Duel, e: EffectId, te: EffectId) -> bool {
    te.get_owner(duel) != e.get_owner(duel)
}
