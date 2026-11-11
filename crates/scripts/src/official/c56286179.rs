// Drill Synchron
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 56286179;

// Three on the field with Dark Blade, a Warrior, attacking Psychic Kappa, a Normal Monster.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (11321183, MZONE),
    (11321183, MZONE),
    (7892180, MZONE | OPPONENT),
    (7892180, MZONE | OPPONENT),
    (7892180, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_PIERCE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_race(duel, RACE_WARRIOR));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EVENT_BATTLE_DAMAGE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, drcon);
    e2.set_target(duel, stock::draw_target::<1>);
    e2.set_operation(duel, stock::draw);
    c.register_effect(duel, e2);
}

fn drcon(duel: &Duel, a: &Args) -> bool {
    if a.ep == a.tp {
        return false;
    }
    let (at, d) = (duel.get_attacker(), duel.get_attack_target());
    d.is_some_and(|d| d.is_defense_pos(duel)) && at.is_some_and(|at| at.is_controler(duel, a.tp) && at.is_race(duel, RACE_WARRIOR))
}
