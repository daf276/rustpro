// Lunalight Leo Dancer
use super::c81196066::LUNALIGHT_MATERIAL;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 24550676;

pub const LISTED_SERIES: &[u16] = &[SET_LUNALIGHT];

const LUNALIGHT_PANTHER_DANCER: u32 = 97165977;
const LUNALIGHT_BLACK_SHEEP: u32 = 11317977;
const LUNALIGHT_SILVER_HOUND: u32 = 35763582;
const SANGAN: u32 = 26202165;
const DARK_HOLE: u32 = 53129443;
pub const MATERIAL_SETCODE: &[u16] = &[SET_LUNALIGHT];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = false;
    const INSF: bool = false;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Code(LUNALIGHT_PANTHER_DANCER),
        LUNALIGHT_MATERIAL,
        LUNALIGHT_MATERIAL,
    ];
}

// In the Extra Deck, Fusion Summoned by Polymerization with Lunalight Panther Dancer and two Lunalights, and on the field
// to attack the opponent's Special Summoned Sangan (revived by their Monster Reborn).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (LUNALIGHT_PANTHER_DANCER, MZONE),
    (LUNALIGHT_BLACK_SHEEP, HAND),
    (LUNALIGHT_SILVER_HOUND, HAND),
    (SANGAN, MZONE | OPPONENT),
    (SANGAN, GRAVE | OPPONENT),
    (83764718, HAND | OPPONENT),
    (DARK_HOLE, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e0.set_value(duel, Value::SpSummonCondition(aux::fuslimit));
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e2.set_value(duel, Value::EffectPlayer(|duel, e, _, rp| rp != e.get_handler_player(duel)));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_EXTRA_ATTACK);
    e3.set_value(duel, 1);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 0));
    e4.set_category(duel, CATEGORY_DESTROY);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_code(duel, EVENT_DAMAGE_STEP_END);
    e4.set_count_limit(duel, 1, 0);
    e4.set_condition(duel, |duel, a| {
        duel.get_attacker() == Some(a.e.get_handler(duel)) && duel.get_attack_target().is_some()
    });
    e4.set_target(duel, target);
    e4.set_operation(duel, operation);
    c.register_effect(duel, e4);
}

fn special_summoned(duel: &Duel, tp: u8) -> CardSet {
    duel.get_matching_group(
        |c, duel| c.is_summon_type(duel, &[SUMMON_TYPE_SPECIAL]),
        tp,
        0,
        LOCATION_MZONE,
        None,
    )
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(
            |c, duel| c.is_summon_type(duel, &[SUMMON_TYPE_SPECIAL]),
            tp,
            0,
            LOCATION_MZONE,
            1,
            None,
        ));
    }
    let g = special_summoned(duel, tp);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = special_summoned(duel, a.tp);
    duel.destroy(g, REASON_EFFECT)?;
    Ok(())
}
