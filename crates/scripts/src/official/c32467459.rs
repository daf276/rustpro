// El Shaddoll Meshahrail
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 32467459;

pub const LISTED_SERIES: &[u16] = &[SET_SHADDOLL, SET_VOID];

const SHADDOLL_CORE: u32 = 4904633;

pub const MATERIAL_SETCODE: &[u16] = &[SET_SHADDOLL];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_SHADDOLL], Some(fc), sumtype, tp)),
        FusionMaterial::Filter(matfilter::<ATTRIBUTE_DARK>),
        FusionMaterial::Filter(matfilter::<ATTRIBUTE_EARTH>),
    ];
}

// In the Extra Deck, Fusion Summoned by Polymerization with Nehshaddoll Genius (Shaddoll, EARTH) and Dark Magician, and on the
// field, with Shaddoll cards in the Deck to search and in the Graveyard to summon, and Dark Hole to send it there.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (66675911, HAND),
    (66675911, HAND),
    (CARD_DARK_MAGICIAN, HAND),
    (66675911, DECK),
    (66675911, GRAVE),
    (53129443, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    c.add_must_be_fusion_summoned(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::Effect(immval));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_cost(duel, Cost::pay_lp::<800>);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

pub fn matfilter<const ATTRIBUTE: u32>(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_attribute_full(duel, ATTRIBUTE, Some(fc), sumtype, tp) || c.is_has_effect(duel, SHADDOLL_CORE).is_some()
}

fn immval(duel: &Duel, e: EffectId, te: EffectId) -> bool {
    if !(te.get_owner_player(duel) != e.get_handler_player(duel) && te.is_activated(duel)) {
        return false;
    }
    if te.is_spell_trap_effect(duel) {
        return true;
    }
    let tc = te.get_handler(duel);
    let lv = e.get_handler(duel).get_level(duel);
    if tc.has_level(duel) {
        tc.get_original_level(duel) < lv
    } else if tc.has_rank(duel) {
        tc.get_original_rank(duel) < lv
    } else {
        false
    }
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_set_card(duel, &[SET_SHADDOLL]) || (c.is_set_card(duel, &[SET_VOID]) && c.is_spell_trap(duel))) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_SHADDOLL]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_GRAVE)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_GRAVE, POS_FACEUP)
}
