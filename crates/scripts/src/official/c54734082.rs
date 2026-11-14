// Magnet Induction
use super::c52566270::face_up_on_field;
use crate::prelude::*;
use crate::testing::{DECK, HAND, MZONE};

const ID: u32 = 54734082;

pub const LISTED_SERIES: &[u16] = &[SET_MAGNET_WARRIOR, SET_MAGNA_WARRIOR];

// In the hand with Alpha The Magnet Warrior on the field, and Beta and Gamma in the Deck to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (99785935, MZONE),
    (99785935, MZONE),
    (39256679, DECK),
    (11549357, DECK),
    (39256679, DECK),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_MAGNET_WARRIOR]) && c.get_original_level(duel) <= 4
}

fn condition(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(cfilter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_MAGNET_WARRIOR])
        && c.is_level_below(duel, 4)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && !face_up_on_field(c, duel, tp)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    stock::special_summon_target(duel, a, chk, |c, duel| filter(c, duel, e, tp), LOCATION_DECK)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    e1.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_MAGNET_WARRIOR, SET_MAGNA_WARRIOR]));
    e1.set_value(duel, 1);
    duel.register_effect(e1, tp);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e2.set_value(duel, Value::EffectPlayer(aux::indoval));
    duel.register_effect(e2, tp);
    stock::special_summon(duel, a, |c, duel| filter(c, duel, e, tp), LOCATION_DECK, POS_FACEUP)
}
