// Mara of the Nordic Alfar
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 73417207;

pub const LISTED_SERIES: &[u16] = &[SET_NORDIC];
// On the field as the Tuner (Level 2) with Garmr and Tyr (Level 4) in the hand for Fabled Levilazebul (Level 10).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (88283496, HAND),
    (88283496, HAND),
    (2333365, HAND),
    (2333365, HAND),
    (21281085, EXTRA),
    (21281085, EXTRA),
    (21281085, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::MaterialFrom(MZONE)];

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
    e1.set_synchro_material_operation(duel, |_, _, _, _, sg, _, _, _| (sg.len() == 3, false));
    c.register_effect(duel, e1);
}

fn synfilter(duel: &Duel, e: EffectId, c: CardId) -> i32 {
    i32::from(c.is_location(duel, LOCATION_HAND) && c.is_set_card(duel, &[SET_NORDIC]) && c.is_controler(duel, e.get_handler_player(duel)))
}

fn synval(duel: &mut Duel, e: EffectId, c: CardId, _: CardId) -> bool {
    if c.is_set_card(duel, &[SET_NORDIC]) && c.is_location(duel, LOCATION_HAND) {
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

fn filterchk(c: CardId, duel: &Duel) -> bool {
    if c.is_set_card(duel, &[SET_NORDIC]) {
        return false;
    }
    c.is_has_effect(duel, EFFECT_HAND_SYNCHRO).is_none()
        || c.is_has_effect(duel, EFFECT_HAND_SYNCHRO + EFFECT_SYNCHRO_CHECK).is_some()
        || c.get_card_effect(duel, EFFECT_HAND_SYNCHRO)
            .first()
            .is_some_and(|&e| e.get_label(duel) != i64::from(ID))
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
    let Some(c) = c else { return (sg.len() < 3, none, none) };
    let others = Some(c.into());
    if sg.len() >= 3
        || !(tg.is_exists(duel, chk2, 1, others) || ntg.is_exists(duel, chk2, 1, others) || sg.is_exists(duel, chk2, 1, others))
    {
        return (false, none, none);
    }
    (true, tg.filter(duel, filterchk, None), ntg.filter(duel, filterchk, None))
}
