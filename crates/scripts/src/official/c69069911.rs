// Kachi Kochi Dragon
use super::c71594310::Procedure;
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 69069911;

// On the field as the only attacker, with a monster attached by Xyz Import, and weak monsters of the opponent's (Key Mace) to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (14602126, HAND),
    (14602126, HAND),
    (14602126, HAND),
    (1929294, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure<4, 2>>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BATTLE_DESTROYING);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, atcon);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_operation(duel, |duel, _| {
        duel.chain_attack();
        Ok(())
    });
    c.register_effect(duel, e1);
}

fn atcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let bc = c.get_battle_target(duel);
    unported!(duel, bc.is_none());
    bc.is_some_and(|bc| bc.is_location(duel, LOCATION_GRAVE) && bc.is_monster(duel))
        && duel.check_mut(|duel| c.can_chain_attack(duel))
        && c.is_status(duel, STATUS_OPPO_BATTLE)
}
