// Dragunity Pilum
use super::c25988873::{spop, sptg};
use crate::prelude::*;
use crate::testing::{HAND, MZONE};

const ID: u32 = 52977572;

pub const LISTED_SERIES: &[u16] = &[SET_DRAGUNITY];
// Copies in hand to Normal Summon, and Dragunity Tribus (Winged Beast) in hand to Special Summon and equip to.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (ID, MZONE), (81962318, HAND), (81962318, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_EQUIP);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_EQUIP);
    e2.set_code(duel, EFFECT_DIRECT_ATTACK);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_EQUIP);
    e3.set_code(duel, EFFECT_CHANGE_BATTLE_DAMAGE);
    e3.set_condition(duel, rdcon);
    e3.set_value(duel, Value::BattleDamage(aux::change_battle_damage::<1, HALF_DAMAGE>));
    c.register_effect(duel, e3);
}

fn rdcon(duel: &Duel, a: &Args) -> bool {
    let (e, tp) = (a.e, a.e.get_handler_player(duel));
    let Some(c) = e.get_handler(duel).get_equip_target(duel) else {
        return false;
    };
    duel.get_attack_target().is_none()
        && c.get_effect_count(duel, EFFECT_DIRECT_ATTACK) < 2
        && duel.get_field_group_count(tp, 0, LOCATION_MZONE) > 0
}
