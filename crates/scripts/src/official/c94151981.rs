// Xyz Armor Torpedo
use engine::constants::MSG_EQUIP;

use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 94151981;

// A second copy in the GY, for Full-Armored Xyz to equip it.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, GRAVE), (39733924, HAND), (39733924, HAND), (6983839, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Message(MSG_EQUIP)];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(3);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).get_overlay_count(duel) == 0);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, Cost::detach_from_self::<2, 2>);
    e2.set_target(duel, stock::draw_target::<1>);
    e2.set_operation(duel, stock::draw);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_target_range(duel, 0, 1);
    e3.set_condition(duel, btlcond);
    e3.set_value(duel, 1);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD);
    e4.set_code(duel, EFFECT_DISABLE);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_target_range(duel, 0, LOCATION_MZONE);
    e4.set_condition(duel, btlcond);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_EQUIP);
    e5.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e5.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e5.set_condition(duel, |duel, a| {
        a.e.get_handler(duel)
            .get_equip_target(duel)
            .is_some_and(|ec| ec.is_type(duel, TYPE_XYZ))
    });
    e5.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e5);
    let e6 = duel.create_effect(c);
    e6.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e6.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e6.set_code(duel, EVENT_BE_BATTLE_TARGET);
    e6.set_range(duel, LOCATION_SZONE);
    e6.set_condition(duel, |duel, a| a.e.get_handler(duel).get_equip_target(duel).is_some());
    e6.set_operation(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        duel.adjust_instantly(Some(c));
        Ok(())
    });
    c.register_effect(duel, e6);
}

fn btlcond(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel)
        .get_equip_target(duel)
        .is_some_and(|ec| duel.get_attacker() == Some(ec) || duel.get_attack_target() == Some(ec))
}
