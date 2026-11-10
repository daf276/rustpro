// Inspector Boarder
use engine::constants::MSG_ATTACK;

use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 15397015;
const EXTRA_DECK_TYPES: u32 = TYPE_FUSION + TYPE_RITUAL + TYPE_SYNCHRO + TYPE_XYZ + TYPE_PENDULUM + TYPE_LINK;

// On the field for both players' monster effects (Elfnote Lucina) and an attack into the opponent's monsters.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (13597785, HAND),
    (13597785, HAND | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_CANNOT_SUMMON);
    e0.set_condition(duel, |duel, a| {
        duel.get_field_group_count(a.e.get_handler_player(duel), LOCATION_MZONE, 0) > 0
    });
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_value(duel, Value::SpSummonCondition(sumlimit));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_FIELD);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_operation(duel, aclimit1);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_FIELD);
    e3.set_code(duel, EVENT_CHAIN_NEGATED);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_operation(duel, aclimit2);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD);
    e4.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e4.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_target_range(duel, 1, 0);
    e4.set_condition(duel, |duel, a| econ(duel, a, ID));
    e4.set_value(duel, Value::EffectPlayer(elimit));
    c.register_effect(duel, e4);
    let e5 = e2.clone_effect(duel);
    e5.set_operation(duel, aclimit3);
    c.register_effect(duel, e5);
    let e6 = e3.clone_effect(duel);
    e6.set_operation(duel, aclimit4);
    c.register_effect(duel, e6);
    let e7 = e4.clone_effect(duel);
    e7.set_condition(duel, |duel, a| econ(duel, a, ID + 1));
    e7.set_target_range(duel, 0, 1);
    c.register_effect(duel, e7);
}

fn sumlimit(duel: &Duel, _: EffectId, _: Option<EffectId>, sp: u8, _: u32, _: u8, _: u8) -> bool {
    duel.get_field_group_count(sp, LOCATION_MZONE, 0) == 0
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, EXTRA_DECK_TYPES)
}

fn typecount(c: CardId, duel: &Duel) -> i64 {
    i64::from(c.get_type(duel) & EXTRA_DECK_TYPES)
}

fn flag_effect_register(duel: &mut Duel, a: &Args, own: bool, flag: u32) {
    if (a.ep == a.tp) == own && a.re.is_some_and(|re| re.is_monster_effect(duel)) {
        let reset = RESET_EVENT | RESETS_STANDARD_DISABLE | RESET_CONTROL | RESET_PHASE | PHASE_END;
        a.e.get_handler(duel).register_flag_effect(duel, flag, reset, 0, 1);
    }
}

fn aclimit1(duel: &mut Duel, a: &Args) -> Script<()> {
    flag_effect_register(duel, a, true, ID);
    Ok(())
}

fn aclimit3(duel: &mut Duel, a: &Args) -> Script<()> {
    flag_effect_register(duel, a, false, ID + 1);
    Ok(())
}

fn aclimit2(duel: &mut Duel, a: &Args) -> Script<()> {
    if a.ep == a.tp && a.re.is_some_and(|re| re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)) {
        a.e.get_handler(duel).reset_flag_effect(duel, ID);
    }
    Ok(())
}

fn aclimit4(duel: &mut Duel, a: &Args) -> Script<()> {
    if a.ep != a.tp && a.re.is_some_and(|re| re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)) {
        a.e.get_handler(duel).reset_flag_effect(duel, ID + 1);
    }
    Ok(())
}

fn econ(duel: &Duel, a: &Args, flag: u32) -> bool {
    let g = duel.get_matching_group(cfilter, a.e.get_handler_player(duel), LOCATION_MZONE, LOCATION_MZONE, None);
    a.e.get_handler(duel).get_flag_effect(duel, flag) >= g.get_class_count(duel, typecount)
}

fn elimit(duel: &Duel, _: EffectId, re: EffectId, _: u8) -> bool {
    re.is_monster_effect(duel)
}
