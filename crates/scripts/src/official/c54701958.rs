// Lunalight Liger Dancer
use super::c81196066::LUNALIGHT_MATERIAL;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 54701958;
const LUNALIGHT_LEO_DANCER: u32 = 24550676;
const LUNALIGHT_BLACK_SHEEP: u32 = 11317977;
const LUNALIGHT_SILVER_HOUND: u32 = 35763582;
const LUNALIGHT_GOLD_LEO: u32 = 8379983;
const SANGAN: u32 = 26202165;
const MONSTER_REBORN: u32 = 83764718;
pub const MATERIAL_SETCODE: &[u16] = &[SET_LUNALIGHT];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = false;
    const INSF: bool = false;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Code(LUNALIGHT_LEO_DANCER),
        LUNALIGHT_MATERIAL,
        LUNALIGHT_MATERIAL,
        LUNALIGHT_MATERIAL,
    ];
}

pub const LISTED_NAMES: &[u32] = &[LUNALIGHT_LEO_DANCER];

// In the Extra Deck, Fusion Summoned by Polymerization with Lunalight Leo Dancer and three Lunalights, and on the field
// with the opponent's Sangan revived by Monster Reborn to destroy, and a Lunalight in the Extra Deck to send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (LUNALIGHT_LEO_DANCER, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (LUNALIGHT_LEO_DANCER, MZONE),
    (LUNALIGHT_BLACK_SHEEP, HAND),
    (LUNALIGHT_SILVER_HOUND, HAND),
    (LUNALIGHT_GOLD_LEO, HAND),
    (MONSTER_REBORN, HAND | OPPONENT),
    (SANGAN, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    c.add_must_be_fusion_summoned(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::Effect(|duel, _, te| !te.is_card_setcode(duel, SET_LUNALIGHT)));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_EXTRA_ATTACK);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e3.set_cost(duel, descost);
    e3.set_target(duel, destg);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
}

fn descostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_LUNALIGHT]) && c.is_able_to_grave_as_cost(duel)
}

fn descost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(descostfilter, tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, descostfilter, tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(CardId::is_special_summoned, tp, 0, LOCATION_MZONE, None);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(CardId::is_special_summoned, a.tp, 0, LOCATION_MZONE, None);
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
