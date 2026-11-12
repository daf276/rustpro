// Yamatako Orochi
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 4632019;

// On the field as the Level 1 Tuner of a Synchro Summon of Red Dragon Archfiend with Zoa, a Level 7 Normal Monster.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (24311372, MZONE),
    (CARD_RED_DRAGON_ARCHFIEND, EXTRA),
    (CARD_RED_DRAGON_ARCHFIEND, EXTRA),
    (CARD_RED_DRAGON_ARCHFIEND, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::MaterialFrom(MZONE), Check::SpecialSummonedCode(CARD_RED_DRAGON_ARCHFIEND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SYNCHRO_LEVEL);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_ONFIELD);
    e1.set_value(duel, Value::Card(|duel, e, _| 8 << 16 | e.get_handler(duel).get_level(duel) as i32));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_BE_MATERIAL);
    e2.set_condition(duel, effcon);
    e2.set_operation(duel, effop);
    c.register_effect(duel, e2);
}

fn effcon(duel: &Duel, a: &Args) -> bool {
    a.r == REASON_SYNCHRO && a.e.get_handler(duel).get_reason_card(duel).is_some_and(|rc| rc.has_level(duel))
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(rc) = c.get_reason_card(duel) else { return Ok(()) };
    if rc.get_original_level(duel) <= 8 {
        let e1 = duel.create_effect(rc);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, 800);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        rc.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
        rc.register_effect(duel, e2);
    } else {
        let e1 = duel.create_effect(rc);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_PIERCE);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        rc.register_effect(duel, e1);
    }
    if !rc.is_type(duel, TYPE_EFFECT) {
        let e3 = duel.create_effect(c);
        e3.set_type(duel, EFFECT_TYPE_SINGLE);
        e3.set_code(duel, EFFECT_ADD_TYPE);
        e3.set_value(duel, TYPE_EFFECT as i32);
        e3.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        rc.register_effect_full(duel, e3, true);
    }
    rc.register_flag_effect_full(
        duel,
        0,
        RESET_EVENT | RESETS_STANDARD,
        EFFECT_FLAG_CLIENT_HINT,
        1,
        0,
        aux::stringid(ID, 0),
    );
    Ok(())
}
