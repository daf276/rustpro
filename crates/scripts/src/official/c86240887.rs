// Buster Blader, the Dragon Destroyer Swordsman
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};
use engine::constants::MSG_ATTACK;

const ID: u32 = 86240887;

pub const MATERIAL_SETCODE: &[u16] = &[SET_BUSTER_BLADER];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(CARD_BUSTER_BLADER), FusionMaterial::Filter(dragon)];
}

// In the Extra Deck, Fusion Summoned with Polymerization from Buster Blader and Luster Dragon #2, with opponent's Dragons.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (CARD_POLYMERIZATION, HAND),
    (CARD_BUSTER_BLADER, MZONE),
    (CARD_BUSTER_BLADER, HAND),
    (17658803, MZONE),
    (17658803, HAND),
    (17658803, MZONE | OPPONENT),
];
// Its effects are all continuous: it is checked by its summon and attacks.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::Message(MSG_ATTACK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CANNOT_DIRECT_ATTACK);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, Value::Card(val));
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE);
    e4.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e4.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e4.set_value(duel, Value::SpSummonCondition(aux::fuslimit));
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_FIELD);
    e5.set_code(duel, EFFECT_SET_POSITION);
    e5.set_range(duel, LOCATION_MZONE);
    e5.set_target_card(duel, |duel, _, c| c.is_race(duel, RACE_DRAGON));
    e5.set_target_range(duel, 0, LOCATION_MZONE);
    e5.set_value(duel, POS_FACEUP_DEFENSE as i32);
    c.register_effect(duel, e5);
    let e6 = duel.create_effect(c);
    e6.set_type(duel, EFFECT_TYPE_FIELD);
    e6.set_range(duel, LOCATION_MZONE);
    e6.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e6.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e6.set_target_range(duel, 0, 1);
    e6.set_value(
        duel,
        Value::EffectPlayer(|duel, _, re, _| re.get_handler(duel).is_race(duel, RACE_DRAGON) && re.is_monster_effect(duel)),
    );
    c.register_effect(duel, e6);
    let e7 = duel.create_effect(c);
    e7.set_type(duel, EFFECT_TYPE_SINGLE);
    e7.set_code(duel, EFFECT_PIERCE);
    c.register_effect(duel, e7);
}

fn dragon(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_race_full(duel, RACE_DRAGON, Some(fc), sumtype, tp)
}

fn val(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    duel.get_matching_group_count(filter, c.get_controler(duel), 0, LOCATION_GRAVE | LOCATION_MZONE, None) as i32 * 1000
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_DRAGON) && (c.is_location(duel, LOCATION_GRAVE) || c.is_faceup(duel))
}
