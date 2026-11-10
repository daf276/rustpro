// Ring of Magnetism
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

struct Procedure;

impl EquipProcedure for Procedure {
    const P: Option<u8> = Some(0);
    const EQLIMIT: Option<fn(&Duel, EffectId, CardId) -> i32> = Some(eqlimit);
}

// In the hand, to equip to the player's own Sangan and draw the opponent's Sangans and Elfnote Lucina into attacking it.
pub const SUPPORT: &[(u32, u16)] = &[
    (20436034, HAND),
    (26202165, MZONE),
    (26202165, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Equip::add_procedure::<Procedure>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_EQUIP);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_value(duel, -500);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_EQUIP);
    e4.set_code(duel, EFFECT_ONLY_BE_ATTACKED);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_FIELD);
    e5.set_code(duel, EFFECT_ONLY_ATTACK_MONSTER);
    e5.set_range(duel, LOCATION_SZONE);
    e5.set_target_range(duel, 0, LOCATION_MZONE);
    e5.set_condition(duel, |duel, a| a.e.get_handler(duel).get_equip_target(duel).is_some());
    c.register_effect(duel, e5);
}

fn eqlimit(duel: &Duel, e: EffectId, c: CardId) -> i32 {
    i32::from(e.get_handler_player(duel) == c.get_controler(duel))
}
