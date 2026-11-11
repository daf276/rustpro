// Mecha Phantom Beast Blue Impala
use super::c72291078::{indcon, spop, sptg};
use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE, OPPONENT};

const ID: u32 = 67489919;
const HAND_CHECK: u32 = EFFECT_HAND_SYNCHRO + EFFECT_SYNCHRO_CHECK;

pub const LISTED_NAMES: &[u32] = &[TOKEN_MECHA_PHANTOM_BEAST];

// In the Graveyard, with weak monsters on the opponent's side and none on its own. The hand Synchro material effect needs a
// non-Tuner Mecha Phantom Beast, and none is ported.
pub const SUPPORT: &[(u32, u16)] = &[(ID, GRAVE), (ID, GRAVE), (27125110, MZONE | OPPONENT), (27125110, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::SpecialSummonedCode(TOKEN_MECHA_PHANTOM_BEAST),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_condition(duel, indcon);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_condition(duel, spcon);
    e3.set_cost(duel, Cost::self_banish);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e4.set_type(duel, EFFECT_TYPE_SINGLE);
    e4.set_code(duel, EFFECT_SYNCHRO_MAT_RESTRICTION);
    e4.set_value(duel, Value::Card(synfilter));
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_SINGLE);
    e5.set_code(duel, EFFECT_CANNOT_BE_SYNCHRO_MATERIAL);
    e5.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e5.set_value(duel, Value::Card(synlimit));
    c.register_effect(duel, e5);
    let e6 = duel.create_effect(c);
    e6.set_type(duel, EFFECT_TYPE_SINGLE);
    e6.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e6.set_code(duel, EFFECT_HAND_SYNCHRO);
    e6.set_label(duel, ID.into());
    e6.set_value(duel, Value::HandSynchro(synval));
    c.register_effect(duel, e6);
}

fn synfilter(duel: &Duel, e: EffectId, c: CardId) -> i32 {
    i32::from(
        c.is_set_card(duel, &[SET_MECHA_PHANTOM_BEAST])
            && c.is_controler(duel, e.get_handler_player(duel))
            && (c.is_location(duel, LOCATION_HAND) || (c.is_faceup(duel) && c.is_location(duel, LOCATION_MZONE))),
    )
}

fn synval(duel: &mut Duel, e: EffectId, c: CardId, _: CardId) -> bool {
    if !c.is_location(duel, LOCATION_HAND) {
        return false;
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, HAND_CHECK);
    e1.set_label(duel, ID.into());
    e1.set_synchro_check_target(duel, synchktg);
    c.register_effect(duel, e1);
    true
}

fn chk2(c: CardId, duel: &Duel) -> bool {
    c.is_has_effect(duel, EFFECT_HAND_SYNCHRO).is_some()
        && c.is_has_effect(duel, HAND_CHECK).is_none()
        && c.get_card_effect(duel, EFFECT_HAND_SYNCHRO)
            .iter()
            .any(|&te| te.get_label(duel) == i64::from(ID))
}

fn synchktg(
    duel: &Duel,
    _e: EffectId,
    c: Option<CardId>,
    sg: CardSet,
    tg: CardSet,
    ntg: CardSet,
    _tsg: CardSet,
    _ntsg: CardSet,
) -> (bool, CardSet, CardSet) {
    let none = CardSet::default();
    let Some(c) = c else { return (true, none, none) };
    let others = Some(c.into());
    let res = tg.is_exists(duel, chk2, 1, others) || ntg.is_exists(duel, chk2, 1, others) || sg.is_exists(duel, chk2, 1, others);
    (res, none, none)
}

fn synlimit(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    i32::from(!c.is_race(duel, RACE_MACHINE))
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    duel.get_field_group_count(tp, LOCATION_MZONE, 0) == 0 && duel.get_field_group_count(tp, 0, LOCATION_MZONE) > 0
}
