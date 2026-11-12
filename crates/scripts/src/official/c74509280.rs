// Malefic Parallel Gear
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 74509280;

// On the field as the Tuner (Level 2) with Malefic Paradox Gear (Level 1) in the hand for Hi-Speedroid Cork Shooter (Level 3).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (48829461, HAND),
    (48829461, HAND),
    (23361526, EXTRA),
    (23361526, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::MaterialFrom(MZONE), Check::SpecialSummonedCode(23361526)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e3.set_code(duel, EFFECT_HAND_SYNCHRO);
    e3.set_label(duel, ID.into());
    e3.set_value(duel, Value::HandSynchro(synval));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e4.set_type(duel, EFFECT_TYPE_SINGLE);
    e4.set_code(duel, EFFECT_SYNCHRO_MAT_RESTRICTION);
    e4.set_value(duel, Value::Card(synfilter));
    c.register_effect(duel, e4);
    let e1 = duel.create_effect(c);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SYNCHRO_MATERIAL_CUSTOM);
    e1.set_synchro_material_operation(duel, |_, _, _, _, sg, _, _, _| (sg.len() == 2, false));
    c.register_effect(duel, e1);
}

fn synfilter(duel: &Duel, e: EffectId, c: CardId) -> i32 {
    i32::from(c.is_location(duel, LOCATION_HAND) && c.is_set_card(duel, &[SET_MALEFIC]) && c.is_controler(duel, e.get_handler_player(duel)))
}

fn synval(duel: &mut Duel, e: EffectId, c: CardId, _: CardId) -> bool {
    if c.is_set_card(duel, &[SET_MALEFIC]) && c.is_location(duel, LOCATION_HAND) {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_HAND_SYNCHRO + EFFECT_SYNCHRO_CHECK);
        e1.set_label(duel, ID.into());
        e1.set_synchro_check_target(duel, synchktg);
        c.register_effect(duel, e1);
        true
    } else {
        false
    }
}

fn chk2(c: CardId, duel: &Duel) -> bool {
    if c.is_has_effect(duel, EFFECT_HAND_SYNCHRO).is_none() || c.is_has_effect(duel, EFFECT_HAND_SYNCHRO + EFFECT_SYNCHRO_CHECK).is_some() {
        return false;
    }
    c.get_card_effect(duel, EFFECT_HAND_SYNCHRO)
        .iter()
        .any(|&e| e.get_label(duel) == i64::from(ID))
}

fn synchktg(
    duel: &Duel,
    _: EffectId,
    c: Option<CardId>,
    sg: CardSet,
    tg: CardSet,
    ntg: CardSet,
    _: CardSet,
    _: CardSet,
) -> (bool, CardSet, CardSet) {
    let none = CardSet::default();
    let Some(c) = c else { return (sg.len() < 2, none, none) };
    let others = Some(c.into());
    if sg.len() >= 2
        || !(tg.is_exists(duel, chk2, 1, others) || ntg.is_exists(duel, chk2, 1, others) || sg.is_exists(duel, chk2, 1, others))
    {
        return (false, none, none);
    }
    (
        true,
        tg.difference(tg.filter(duel, chk2, None)),
        ntg.difference(ntg.filter(duel, chk2, None)),
    )
}
