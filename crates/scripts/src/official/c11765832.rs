// Garura, Wings of Resonant Life
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};

const ID: u32 = 11765832;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::GroupFilter(ffilter); 2];
}

// In the Extra Deck, Fusion Summoned by Polymerization with Mystical Elf and Maiden of the Moonlight.
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (24094653, HAND), (15025844, HAND), (79629370, HAND)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CHANGE_BATTLE_DAMAGE);
    e1.set_value(duel, Value::BattleDamage(aux::change_battle_damage::<1, DOUBLE_DAMAGE>));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, stock::draw_target::<1>);
    e2.set_operation(duel, stock::draw);
    c.register_effect(duel, e2);
}

fn ffilter(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, sump: u8, sg: Option<CardSet>) -> bool {
    let Some(sg) = sg else { return true };
    let others = Some(c.into());
    sg.filter_count(duel, aux::TRUE, others) == 0
        || (sg.is_exists(
            duel,
            |sc, duel| sc.is_attribute_full(duel, c.get_attribute(duel), Some(fc), sumtype, sump),
            1,
            others,
        ) && sg.is_exists(
            duel,
            |sc, duel| sc.is_race_full(duel, c.get_race(duel), Some(fc), sumtype, sump),
            1,
            others,
        ) && !sg.is_exists(duel, |sc, duel| fusfilter(sc, duel, c.get_code(duel), fc, sumtype, sump), 1, others))
}

fn fusfilter(c: CardId, duel: &Duel, code: u32, fc: CardId, sumtype: u64, sump: u8) -> bool {
    c.is_summon_code(duel, Some(fc), sumtype, sump, &[code]) && c.is_has_effect(duel, 511002961).is_none()
}
