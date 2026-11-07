// Vision HERO Trinity
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 46759931;
pub const MATERIAL_SETCODE: &[u16] = &[SET_HERO];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] =
        &[FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_HERO], Some(fc), sumtype, tp)); 3];
}

// In the Extra Deck and on the field, Fusion Summoned by Polymerization with Elemental HERO Sparkman, Clayman and Avian,
// facing a Sangan to attack twice.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (20721928, HAND),
    (84327329, HAND),
    (21844576, HAND),
    (26202165, MZONE | OPPONENT),
];
// Both effects are continuous and never chain, so nothing resolves.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, regcon);
    e1.set_operation(duel, regop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_EXTRA_ATTACK);
    e2.set_condition(duel, regcon);
    e2.set_value(duel, 2);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_CANNOT_DIRECT_ATTACK);
    c.register_effect(duel, e3);
}

fn regcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_fusion_summoned(duel)
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SET_ATTACK);
    e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END);
    e1.set_value(duel, c.get_base_attack(duel) * 2);
    c.register_effect(duel, e1);
    Ok(())
}
