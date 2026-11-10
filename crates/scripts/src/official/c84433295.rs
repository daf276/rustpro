// Quintet Magician
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 84433295;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = false;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] =
        &[FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_race_full(duel, RACE_SPELLCASTER, Some(fc), sumtype, tp)); 5];
}

// In the Extra Deck, Fusion Summoned by Polymerization with five different Spellcasters (Mystical Elf, Lady of Faith, Eldeen,
// Flame Manipulator, Fairywitch), against the opponent's monsters.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (15025844, HAND),
    (17358176, HAND),
    (6367785, MZONE),
    (34460851, MZONE),
    (37160778, MZONE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    c.add_must_be_fusion_summoned(duel);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_fusion_summoned(duel) && a.e.get_label(duel) == 1
    });
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_MATERIAL_CHECK);
    e3.set_value(duel, Value::MaterialCheck(valcheck));
    e3.set_label_object(duel, Some(e2));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE);
    e4.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_code(duel, EFFECT_UNRELEASABLE_SUM);
    e4.set_value(duel, 1);
    c.register_effect(duel, e4);
    for code in [
        EFFECT_UNRELEASABLE_NONSUM,
        EFFECT_CANNOT_BE_FUSION_MATERIAL,
        EFFECT_INDESTRUCTABLE_EFFECT,
    ] {
        let e = e4.clone_effect(duel);
        e.set_code(duel, code);
        c.register_effect(duel, e);
    }
}

fn valcheck(duel: &mut Duel, e: EffectId, c: CardId) {
    let g = c
        .get_material(duel)
        .filter(duel, |mc, duel| mc.is_race(duel, RACE_SPELLCASTER), None);
    if g.get_class_count(duel, |mc, duel| i64::from(mc.get_code(duel))) == 5
        && let Some(e2) = e.get_label_object(duel)
    {
        e2.set_label(duel, 1);
    }
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_field_group(tp, 0, LOCATION_ONFIELD);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_field_group(a.tp, 0, LOCATION_ONFIELD);
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
