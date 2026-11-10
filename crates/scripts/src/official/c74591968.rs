// Mystic Swordsman LV4
use super::c47507260::{bdop, descon, destg, spcon, spop, sptg};
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE, OPPONENT};

const ID: u32 = 74591968;
const MYSTIC_SWORDSMAN_LV6: u32 = 60482781;

pub const LISTED_NAMES: &[u32] = &[MYSTIC_SWORDSMAN_LV6];
// Mystic Swordsman LV6 in the Deck to level up to, and face-down monsters (Tomozaurus) on the opponent's side to attack and destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (MYSTIC_SWORDSMAN_LV6, DECK),
    (MYSTIC_SWORDSMAN_LV6, DECK),
    (46457856, MZONE | OPPONENT),
    (46457856, MZONE | OPPONENT),
    (46457856, MZONE | OPPONENT),
    (46457856, MZONE | OPPONENT),
    (46457856, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_BATTLE_DESTROYING);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_operation(duel, bdop::<ID>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_BATTLE_START);
    e2.set_condition(duel, descon);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_code(duel, EVENT_PHASE + PHASE_END);
    e3.set_condition(duel, spcon::<ID>);
    e3.set_cost(duel, Cost::self_to_grave);
    e3.set_target(duel, sptg::<MYSTIC_SWORDSMAN_LV6>);
    e3.set_operation(duel, spop::<MYSTIC_SWORDSMAN_LV6>);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE);
    e4.set_code(duel, EFFECT_LIMIT_SUMMON_PROC);
    e4.set_condition(duel, |_, a| a.c.is_none());
    c.register_effect(duel, e4);
}

fn desop(duel: &mut Duel, _: &Args) -> Script<()> {
    if let Some(d) = duel.get_attack_target()
        && d.is_relate_to_battle(duel)
    {
        duel.destroy(d, REASON_EFFECT)?;
    }
    Ok(())
}
