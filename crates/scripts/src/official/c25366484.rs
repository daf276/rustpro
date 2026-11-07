// Elemental HERO Shining Flare Wingman
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 25366484;
pub const MATERIAL_SETCODE: &[u16] = &[SET_HERO, SET_ELEMENTAL_HERO];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(35809262), FusionMaterial::Code(20721928)];
}

// In the Extra Deck, Fusion Summoned by Polymerization with Elemental HERO Flame Wingman on the field and Sparkman, and on the
// field to battle weak Sangan, with an Elemental HERO in the Graveyard for the ATK.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (35809262, MZONE),
    (20721928, HAND),
    (21844576, GRAVE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    c.add_must_be_fusion_summoned(duel);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DAMAGE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EVENT_BATTLE_DESTROYING);
    e2.set_condition(duel, aux::bdgcon);
    e2.set_target(duel, super::c35809262::damtg);
    e2.set_operation(duel, super::c35809262::damop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_code(duel, EFFECT_UPDATE_ATTACK);
    e3.set_value(duel, Value::Card(atkup));
    c.register_effect(duel, e3);
}

fn atkup(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    let filter = |c: CardId, duel: &Duel| c.is_set_card(duel, &[SET_ELEMENTAL_HERO]);
    duel.get_matching_group_count(filter, c.get_controler(duel), LOCATION_GRAVE, 0, None) as i32 * 300
}
