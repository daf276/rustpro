// Cyber End Dragon the Decisive Strike Dragon
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 30275298;
const LIMITER_REMOVAL: u32 = 23171610;

pub const LISTED_NAMES: &[u32] = &[LIMITER_REMOVAL];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Filter(matfilter); 3];
}

// Fusion Summoned by Polymerization with Cyber Dragons (LIGHT Machines with 2100 ATK) and Limiter Removal in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (CARD_CYBER_DRAGON, HAND),
    (CARD_CYBER_DRAGON, MZONE),
    (CARD_CYBER_DRAGON, MZONE),
    (CARD_CYBER_DRAGON, MZONE),
    (LIMITER_REMOVAL, DECK),
    (LIMITER_REMOVAL, DECK),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    c.add_must_be_fusion_summoned(duel);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e0.set_code(duel, EFFECT_MATERIAL_CHECK);
    e0.set_value(duel, Value::MaterialCheck(valcheck));
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND | CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE | EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_label_object(duel, Some(e0));
    e1.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_fusion_summoned(duel) && a.e.get_label_object(duel).is_some_and(|e0| e0.get_label(duel) == 1)
    });
    e1.set_cost(duel, Cost::pay_lp::<4000>);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_PIERCE);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_EXTRA_ATTACK);
    e3.set_condition(duel, atcon);
    e3.set_value(duel, 2);
    c.register_effect(duel, e3);
}

fn matfilter(c: CardId, duel: &Duel, sc: CardId, st: u64, tp: u8) -> bool {
    c.is_race_full(duel, RACE_MACHINE, Some(sc), st, tp) && c.is_attack_above(duel, 2100)
}

fn valcheck(duel: &mut Duel, e: EffectId, c: CardId) {
    let mg = c.get_material(duel);
    let light = mg.filter_count(duel, |c, duel| c.is_attribute(duel, ATTRIBUTE_LIGHT), None);
    e.set_label(duel, i64::from(light == mg.len()));
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, LIMITER_REMOVAL) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn atcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.get_attack(duel) > c.get_base_attack(duel)
}
