// Magical Mid-Breaker Field
use crate::prelude::*;
use crate::testing::HAND;

const ID: u32 = 71650854;

// Copies in the hand to activate.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_condition(duel, condition);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e2.set_property(duel, EFFECT_FLAG_SET_AVAILABLE + EFFECT_FLAG_IGNORE_IMMUNE);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_condition(duel, |duel, _| duel.is_phase(PHASE_MAIN1));
    e2.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e3.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e3.set_value(duel, Value::EffectPlayer(aux::indoval));
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e4.set_target_range(duel, 0, LOCATION_MZONE);
    e4.set_value(
        duel,
        Value::EffectPlayer(|duel, e, re, rp| rp != 1 - e.get_handler_player(duel) && !re.get_handler(duel).is_immune_to_effect(duel, e)),
    );
    c.register_effect(duel, e4);
    let e5 = e3.clone_effect(duel);
    e5.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e5.set_target_range(duel, 0, LOCATION_MZONE);
    e5.set_value(duel, Value::EffectPlayer(|duel, e, _, rp| rp != 1 - e.get_handler_player(duel)));
    c.register_effect(duel, e5);
    let e6 = duel.create_effect(c);
    e6.set_type(duel, EFFECT_TYPE_FIELD);
    e6.set_code(duel, EFFECT_CANNOT_SSET);
    e6.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e6.set_range(duel, LOCATION_FZONE);
    e6.set_target_range(duel, 1, 0);
    e6.set_target_card(duel, |duel, _, c| c.is_type(duel, TYPE_FIELD));
    c.register_effect(duel, e6);
    let e7 = duel.create_effect(c);
    e7.set_type(duel, EFFECT_TYPE_FIELD);
    e7.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e7.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e7.set_range(duel, LOCATION_FZONE);
    e7.set_target_range(duel, 1, 0);
    e7.set_value(
        duel,
        Value::EffectPlayer(|duel, _, re, _| re.is_active_type(duel, TYPE_FIELD) && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)),
    );
    c.register_effect(duel, e7);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(a.tp)
        && duel.is_main_phase()
        && (!duel.check_phase_activity() || (duel.get_flag_effect(a.tp, CARD_MAGICAL_MIDBREAKER) > 0 && duel.get_current_chain() == 1))
}
