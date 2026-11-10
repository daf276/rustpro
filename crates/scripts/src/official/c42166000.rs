// Egyptian God Slime
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 42166000;
const GUARDIAN_SLIME: u32 = 15771991;
const WATER_SPIRIT: u32 = 487395;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_race_full(duel, RACE_AQUA, Some(fc), sumtype, tp)),
        FusionMaterial::Filter(matfilter),
    ];
}

pub const LISTED_NAMES: &[u32] = &[ID];
// In the Extra Deck, Summoned by releasing Guardian Slime (Level 10, 0 ATK) or by Polymerization with Guardian Slime and
// Water Spirit, with attackers on the opponent's side.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (GUARDIAN_SLIME, MZONE),
    (GUARDIAN_SLIME, MZONE),
    (WATER_SPIRIT, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_range(duel, LOCATION_EXTRA);
    e1.set_condition(duel, hspcon);
    e1.set_target(duel, hsptg);
    e1.set_operation(duel, hspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_TRIPLE_TRIBUTE);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e3.set_value(duel, 1);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD);
    e4.set_code(duel, EFFECT_CANNOT_SELECT_BATTLE_TARGET);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_target_range(duel, 0, LOCATION_MZONE);
    e4.set_value(duel, Value::Card(|duel, _, c| i32::from(tglimit(duel, c))));
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_FIELD);
    e5.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e5.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE + EFFECT_FLAG_SET_AVAILABLE);
    e5.set_range(duel, LOCATION_MZONE);
    e5.set_target_range(duel, LOCATION_MZONE, 0);
    e5.set_target_card(duel, |duel, _, c| tglimit(duel, c));
    e5.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e5);
}

fn matfilter(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_attribute_full(duel, ATTRIBUTE_WATER, Some(fc), sumtype, tp) && c.get_level(duel) == 10
}

fn hspfilter(c: CardId, duel: &Duel, tp: u8, sc: CardId) -> bool {
    c.is_race(duel, RACE_AQUA)
        && c.get_level(duel) == 10
        && c.get_attack(duel) == 0
        && duel.get_location_count_from_ex(tp, tp, Some(c.into()), sc) > 0
}

fn release_options(c: CardId, tp: u8) -> ReleaseOptions {
    ReleaseOptions {
        check_field: true,
        to_check: Some(c),
        ..ReleaseOptions::new(tp)
    }
}

fn hspcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.check_release_group_full(tp, |rc, duel| hspfilter(rc, duel, tp, c), 1, release_options(c, tp))
}

fn hsptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let Some(c) = a.c else { return Ok(false) };
    let g = duel.select_release_group_full(tp, |rc, duel| hspfilter(rc, duel, tp, c), (1, 1), true, release_options(c, tp))?;
    if g.is_empty() {
        return Ok(false);
    }
    a.e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn hspop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(g) = a.e.get_label_object_group(duel) {
        duel.release(g, REASON_COST | REASON_MATERIAL)?;
    }
    Ok(())
}

fn tglimit(duel: &Duel, c: CardId) -> bool {
    c.get_code(duel) != ID
}
