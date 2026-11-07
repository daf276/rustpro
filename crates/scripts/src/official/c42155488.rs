// Genomix Fighter
use crate::prelude::*;
use crate::testing::{EXTRA, HAND, MZONE};

const ID: u32 = 42155488;

// A Level 3 Tuner without tribute for Kuibelt the Blade Dragon with Kojikocy, and on the field to declare a race.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, MZONE), (1184620, MZONE), (87837090, EXTRA)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SUMMON_PROC);
    e1.set_condition(duel, ntcon);
    e1.set_operation(duel, ntop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_target(duel, dectg);
    e2.set_operation(duel, decop);
    c.register_effect(duel, e2);
}

fn ntcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    a.min == 0 && c.get_level(duel) > 4 && duel.get_location_count(c.get_controler(duel), LOCATION_MZONE) > 0
}

fn ntop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(c) = a.c else { return Ok(()) };
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD_DISABLE & !RESET_TOFIELD));
    e1.set_code(duel, EFFECT_SET_BASE_ATTACK);
    e1.set_value(duel, 1100);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_CHANGE_LEVEL);
    e2.set_value(duel, 3);
    c.register_effect(duel, e2);
    Ok(())
}

fn dectg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(true);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RACE);
    let rc = duel.announce_race(tp, 1, RACE_ALL)?;
    e.set_label(duel, rc as i64);
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, sumlimit);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    e1.set_label(duel, rc as i64);
    duel.register_effect(e1, tp);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    duel.register_effect(e2, tp);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 1)));
    Ok(false)
}

fn sumlimit(duel: &Duel, e: EffectId, c: CardId) -> bool {
    c.get_race(duel) as i64 != e.get_label(duel)
}

fn decop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if c.is_relate_to_effect(duel, e) && c.is_faceup(duel) {
        let rc = e.get_label(duel);
        c.set_hint(duel, CHINT_RACE, rc as u64);
        let e1 = duel.create_effect(c);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SYNCHRO_CHECK);
        e1.set_value(duel, Value::Card(syncheck));
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        e1.set_label(duel, rc);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn syncheck(duel: &Duel, e: EffectId, c: CardId) -> i32 {
    c.assume_property(duel, ASSUME_RACE, e.get_label(duel) as u64);
    1
}
