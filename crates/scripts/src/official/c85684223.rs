// Reaper on the Nightmare
use engine::constants::MSG_BECOME_TARGET;

use super::c23205979::{desop1, desop2, operation, register, target};
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, OPPONENT};

const ID: u32 = 85684223;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(23205979), FusionMaterial::Code(59290628)];
}

// Fusion Summoned by Polymerization with Spirit Reaper and Nightmare Horse; an opponent's Mage Power to target it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (23205979, HAND),
    (59290628, HAND),
    (83746708, HAND | OPPONENT),
    (83746708, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Message(MSG_BECOME_TARGET),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_HANDES);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_BATTLE_DAMAGE);
    e2.set_condition(duel, |duel, a| a.ep != a.tp && duel.get_attack_target().is_none());
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_code(duel, EFFECT_SELF_DESTROY);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).get_owner_target_count(duel) > 0);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_code(duel, EVENT_CHAIN_SOLVED);
    e4.set_operation(duel, desop1);
    c.register_effect(duel, e4);
    let e5 = e4.clone_effect(duel);
    e5.set_code(duel, EVENT_BATTLED);
    e5.set_operation(duel, desop2);
    c.register_effect(duel, e5);
    let e6 = e4.clone_effect(duel);
    e6.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e6.set_code(duel, EVENT_BECOME_TARGET);
    e6.set_operation(duel, register);
    c.register_effect(duel, e6);
    let e7 = duel.create_effect(c);
    e7.set_type(duel, EFFECT_TYPE_SINGLE);
    e7.set_code(duel, EFFECT_DIRECT_ATTACK);
    c.register_effect(duel, e7);
}
