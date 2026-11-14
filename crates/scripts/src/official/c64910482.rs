// T.G. Cyber Magician
use super::c1315120::regop;
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE, OPPONENT};

const ID: u32 = 64910482;

pub const LISTED_SERIES: &[u16] = &[SET_TG];
// Two on the field as the Level 1 Tuner with Disk Magician (Level 4) for T.G. Hyper Librarian, and destroyed by the opponent's Raigeki
// to search a T.G. monster from the Deck. The hand Synchro material it grants is checked from T.G. Warwolf's file.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (76446915, MZONE),
    (76446915, MZONE),
    (90953320, EXTRA),
    (90953320, EXTRA),
    (90953320, EXTRA),
    (94350039, DECK),
    (12580477, DECK | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::MaterialFrom(MZONE), Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_operation(duel, regop::<ID, 0>);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e3.set_code(duel, EFFECT_HAND_SYNCHRO);
    e3.set_label(duel, ID.into());
    e3.set_value(duel, Value::HandSynchro(synval));
    c.register_effect(duel, e3);
}

fn synval(duel: &mut Duel, e: EffectId, c: CardId, sc: CardId) -> bool {
    if sc.is_set_card(duel, &[SET_TG])
        && (!c.is_type(duel, TYPE_TUNER) || c.is_has_effect(duel, EFFECT_NONTUNER).is_some())
        && c.is_set_card(duel, &[SET_TG])
        && c.is_location(duel, LOCATION_HAND)
    {
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
    let Some(c) = c else { return (true, none, none) };
    let others = Some(c.into());
    let res = tg.is_exists(duel, chk2, 1, others) || ntg.is_exists(duel, chk2, 1, others) || sg.is_exists(duel, chk2, 1, others);
    (res, none, none)
}
