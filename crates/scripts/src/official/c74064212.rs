// Vylon Prism
use super::c1281505::{add_equip_trigger, eqtg};
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};
use engine::constants::MSG_EQUIP;

const ID: u32 = 74064212;

// Two copies and Sangans on the field, with Elfnote Lucinas on the opponent's side to attack, so a Prism is destroyed and
// equipped to a Sangan that then battles.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (26202165, MZONE),
    (26202165, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Message(MSG_EQUIP)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    add_equip_trigger(duel, c, ID, eqtg);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_EQUIP);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_condition(duel, atkcon);
    e2.set_value(duel, 1000);
    c.register_effect(duel, e2);
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    let ph = duel.get_current_phase();
    a.e.get_handler(duel)
        .get_equip_target(duel)
        .is_some_and(|ec| (ph == PHASE_DAMAGE || ph == PHASE_DAMAGE_CAL) && ec.is_relate_to_battle(duel))
}
