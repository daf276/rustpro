// Thousand-Eyes Restrict
use super::c64631466::{add_stat_effects, eqcon, eqop, eqtg};
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 63519819;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(64631466), FusionMaterial::Code(27125110)];
}

// On the field and in the Extra Deck, Fusion Summoned by Polymerization with Relinquished and Thousand-Eyes Idol, with the
// opponent's monsters to equip.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (64631466, HAND),
    (27125110, HAND),
    (27324313, MZONE | OPPONENT),
    (27324313, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_EQUIP);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, eqcon::<ID>);
    e1.set_target(duel, eqtg);
    e1.set_operation(duel, eqop::<ID>);
    c.register_effect(duel, e1);
    aux::add_er_equip_limit(duel, c, Some(eqcon::<ID>), e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_CANNOT_ATTACK);
    e2.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e2.set_target_card(duel, antarget);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_CANNOT_CHANGE_POSITION);
    e3.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    c.register_effect(duel, e3);
    add_stat_effects::<ID>(duel, c);
}

fn antarget(duel: &Duel, e: EffectId, c: CardId) -> bool {
    c != e.get_handler(duel)
}
