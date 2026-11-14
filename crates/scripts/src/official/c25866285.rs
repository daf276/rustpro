// Naturia Stinkbug
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 25866285;

pub const LISTED_SERIES: &[u16] = &[SET_NATURIA];
// Copies on the field to be attacked, and the opponent's attackers.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_BE_BATTLE_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, condition);
    e1.set_cost(duel, Cost::self_to_grave);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    a.eg.and_then(|eg| eg.get_first())
        .is_some_and(|ec| ec.is_faceup(duel) && ec.is_controler(duel, a.tp) && ec.is_set_card(duel, &[SET_NATURIA]))
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.negate_attack()? {
        duel.skip_phase(1 - a.tp, PHASE_BATTLE, RESET_PHASE | PHASE_BATTLE_STEP, 1);
    }
    Ok(())
}
