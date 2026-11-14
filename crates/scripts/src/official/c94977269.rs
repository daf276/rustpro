// El Shaddoll Winda
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND};

const ID: u32 = 94977269;

pub const LISTED_SERIES: &[u16] = &[SET_SHADDOLL];
pub const MATERIAL_SETCODE: &[u16] = &[SET_SHADDOLL];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_SHADDOLL], Some(fc), sumtype, tp)),
        FusionMaterial::Filter(matfilter),
    ];
}

// In the Extra Deck, Fusion Summoned by Polymerization with two DARK Shaddoll monsters, sent to the Graveyard by Dark Hole
// drawn later, with a Shaddoll Spell in the Graveyard to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (24094653, HAND),
    (30328508, HAND),
    (3717252, HAND),
    (53129443, DECK),
    (53129443, DECK),
    (44394295, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    duel.enable_global_flag(GLOBALFLAG_SPSUMMON_COUNT);
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE + EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e2.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e2.set_range(duel, LOCATION_EXTRA);
    e2.set_value(duel, Value::SpSummonCondition(aux::fuslimit));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_value(duel, Value::EffectPlayer(indval));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD);
    e4.set_code(duel, EFFECT_SPSUMMON_COUNT_LIMIT);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e4.set_target_range(duel, 1, 1);
    e4.set_value(duel, 1);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 0));
    e5.set_category(duel, CATEGORY_TOHAND);
    e5.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e5.set_code(duel, EVENT_TO_GRAVE);
    e5.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e5.set_target(duel, thtg);
    e5.set_operation(duel, stock::first_target_to_hand);
    c.register_effect(duel, e5);
}

fn matfilter(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_attribute_full(duel, ATTRIBUTE_DARK, Some(lc), sumtype, tp) || c.is_has_effect(duel, 4904633).is_some()
}

fn indval(duel: &Duel, e: EffectId, _: EffectId, tp: u8) -> bool {
    tp != e.get_handler_player(duel)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SHADDOLL]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && thfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, 0, 0);
    Ok(false)
}
