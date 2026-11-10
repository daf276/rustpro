// Lunalight Sabre Dancer
use super::c81196066::LUNALIGHT_MATERIAL;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 88753594;
const LUNALIGHT_BLACK_SHEEP: u32 = 11317977;
const LUNALIGHT_SILVER_HOUND: u32 = 35763582;
const LUNALIGHT_GOLD_LEO: u32 = 8379983;
pub const MATERIAL_SETCODE: &[u16] = &[SET_LUNALIGHT];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[LUNALIGHT_MATERIAL; 3];
}

// In the Extra Deck, Fusion Summoned by Polymerization with three Lunalights, on the field to gain ATK from the Beast-Warriors
// in the Graveyards, and in the Graveyard to boost a Fusion Monster on the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (ID, GRAVE),
    (CARD_POLYMERIZATION, HAND),
    (LUNALIGHT_BLACK_SHEEP, HAND),
    (LUNALIGHT_SILVER_HOUND, HAND),
    (LUNALIGHT_GOLD_LEO, HAND),
    (LUNALIGHT_GOLD_LEO, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_value(duel, Value::SpSummonCondition(aux::fuslimit));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, Value::Card(atkval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_category(duel, CATEGORY_ATKCHANGE);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e4.set_range(duel, LOCATION_GRAVE);
    e4.set_condition(duel, aux::exccon);
    e4.set_cost(duel, Cost::self_banish);
    e4.set_target(duel, atktg);
    e4.set_operation(duel, atkop);
    e4.set_count_limit(duel, 1, ID);
    c.register_effect(duel, e4);
}

fn atkfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_location(duel, LOCATION_GRAVE) || c.is_faceup(duel)) && c.is_race(duel, RACE_BEASTWARRIOR)
}

fn atkval(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    let location = LOCATION_GRAVE | LOCATION_REMOVED;
    duel.get_matching_group_count(atkfilter, c.get_controler(duel), location, location, None) as i32 * 200
}

fn atkfilter2(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_FUSION)
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && atkfilter2(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(atkfilter2, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, atkfilter2, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && tc.is_faceup(duel)
    {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, 3000);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
