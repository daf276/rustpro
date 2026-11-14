// Elemental HERO Absolute Zero
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 40854197;

pub const LISTED_SERIES: &[u16] = &[SET_HERO];
pub const MATERIAL_SETCODE: &[u16] = &[SET_HERO];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_HERO], Some(fc), sumtype, tp)),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_attribute_full(duel, ATTRIBUTE_WATER, Some(fc), sumtype, tp)),
    ];
}

// In the Extra Deck, Fusion Summoned by Polymerization with the WATER Elemental HERO Ocean and Elemental HERO Stratos,
// and on the field to be destroyed by Dark Hole, with a monster of the opponent's to destroy in turn.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (37195861, HAND),
    (40044918, HAND),
    (53129443, HAND),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    c.add_must_be_fusion_summoned(duel);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_LEAVE_FIELD);
    e2.set_condition(duel, descon);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_code(duel, EFFECT_UPDATE_ATTACK);
    e3.set_value(duel, Value::Card(atkup));
    c.register_effect(duel, e3);
}

fn atkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.get_code(duel) != ID && c.is_attribute(duel, ATTRIBUTE_WATER)
}

fn atkup(duel: &Duel, _: EffectId, _: CardId) -> i32 {
    duel.get_matching_group_count(atkfilter, 0, LOCATION_MZONE, LOCATION_MZONE, None) as i32 * 500
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_position(duel, POS_FACEUP) && c.is_previous_location(duel, LOCATION_ONFIELD)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    let g = duel.get_matching_group(|_, _| true, tp, 0, LOCATION_MZONE, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(|_, _| true, a.tp, 0, LOCATION_MZONE, None);
    duel.destroy(g, REASON_EFFECT)?;
    Ok(())
}
