// Gravekeeper's Shaman
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};
use engine::constants::MSG_CHAIN_DISABLED;

const ID: u32 = 58139128;

pub const LISTED_NAMES: &[u32] = &[CARD_NECROVALLEY];
pub const LISTED_SERIES: &[u16] = &[SET_GRAVEKEEPERS];
// Glow-Up Bulb and Plaguespreader Zombie to activate from the Graveyard, and Necrovalley to activate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (67441435, GRAVE | OPPONENT),
    (33420078, GRAVE | OPPONENT),
    (67441435, GRAVE),
    (CARD_NECROVALLEY, HAND),
];
// No activated effect: its negation of Graveyard monster effects shows as MSG_CHAIN_DISABLED.
pub const CHECKS: &[Check] = &[Check::Message(MSG_CHAIN_DISABLED)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_DEFENSE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::Card(defval));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EVENT_CHAIN_ACTIVATING);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3.set_target_range(duel, 0, 1);
    e3.set_condition(duel, econ);
    e3.set_value(
        duel,
        Value::EffectPlayer(|duel, _, re, _| re.get_handler(duel).is_type(duel, TYPE_FIELD) && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)),
    );
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD);
    e4.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_property(duel, EFFECT_FLAG_IGNORE_RANGE + EFFECT_FLAG_SET_AVAILABLE);
    e4.set_condition(duel, econ);
    e4.set_target_card(duel, |duel, _, c| c.is_type(duel, TYPE_FIELD));
    e4.set_value(
        duel,
        Value::EffectPlayer(|duel, e, re, _| e.get_owner_player(duel) != re.get_owner_player(duel)),
    );
    c.register_effect(duel, e4);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GRAVEKEEPERS]) && c.is_monster(duel)
}

fn defval(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    duel.get_matching_group_count(filter, c.get_controler(duel), LOCATION_GRAVE, 0, None) as i32 * 200
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    let loc: u32 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_LOCATION);
    if re.is_monster_effect(duel) && !re.get_handler(duel).is_set_card(duel, &[SET_GRAVEKEEPERS]) && loc == LOCATION_GRAVE {
        duel.negate_effect(a.ev as u8);
    }
    Ok(())
}

fn econ(duel: &Duel, _: &Args) -> bool {
    duel.is_environment(CARD_NECROVALLEY)
}
