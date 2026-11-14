// Gladiator Beast Laquari
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE, OPPONENT};

const ID: u32 = 78868776;

pub const LISTED_SERIES: &[u16] = &[SET_GLADIATOR];
pub const LISTED_NAMES: &[u32] = &[ID];
// On the field after battle, with Gladiator Beast Hoplomus in the Deck to summon and monsters of the opponent's to fight.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (4253484, DECK),
    (4253484, DECK),
    (4253484, DECK),
    (27125110, MZONE | OPPONENT),
    (27125110, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SET_BASE_ATTACK);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).get_flag_effect(duel, ID) > 0);
    e1.set_value(duel, 2100);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_PHASE | PHASE_BATTLE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).get_battled_group_count(duel) > 0);
    e2.set_cost(duel, Cost::self_to_deck);
    e2.set_target(duel, |duel, a, chk, _| {
        super::c57731460::sptg(duel, a, chk, |c, duel| filter(c, duel, a.e, a.tp))
    });
    e2.set_operation(duel, |duel, a| {
        super::c57731460::spop(duel, a, |c, duel| filter(c, duel, a.e, a.tp), 100, false, 0)
    });
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    !c.is_code(duel, ID) && c.is_set_card(duel, &[SET_GLADIATOR]) && c.is_can_be_special_summoned(duel, e, 100, tp, false, false)
}
