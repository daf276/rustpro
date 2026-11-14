// Infernoid Attondel
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

use super::c60551528::{rmcost, rmop, rmtg};

const ID: u32 = 88301393;

pub const LISTED_SERIES: &[u16] = &[SET_INFERNOID];

// Copies on the field to attack and Tribute, with the opponent's Sangan to destroy by battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Infernoid::register_summon_procedure::<2>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BATTLE_DESTROYING);
    e1.set_condition(duel, atcon);
    e1.set_operation(duel, |duel, _| {
        duel.chain_attack();
        Ok(())
    });
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_count_limit(duel, 1, 0);
    e2.set_cost(duel, rmcost);
    e2.set_target(duel, rmtg);
    e2.set_operation(duel, rmop);
    c.register_effect(duel, e2);
}

fn atcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let Some(bc) = c.get_battle_target(duel) else { return false };
    bc.is_location(duel, LOCATION_GRAVE)
        && bc.is_monster(duel)
        && duel.check_mut(|duel| c.can_chain_attack(duel))
        && c.is_status(duel, STATUS_OPPO_BATTLE)
}
