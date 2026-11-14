// Alba-Lenatus the Abyss Dragon
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE, OPPONENT};

const ID: u32 = 3410461;

pub const LISTED_SERIES: &[u16] = &[SET_FUSION];
pub const LISTED_NAMES: &[u32] = &[CARD_ALBAZ];

struct Materials;

impl FusionMixRep for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const FUN1: FusionMaterial =
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_race_full(duel, RACE_DRAGON, Some(fc), sumtype, tp));
    const MINC: i32 = 1;
    const MAXC: i32 = 99;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(CARD_ALBAZ)];
}

struct Contact;

impl ContactFusion for Contact {
    const SUMCON: Option<Value> = Some(Value::SpSummonCondition(splimit));
    const CANNOT_BE_LIZARD: bool = false;

    fn group(duel: &Duel, tp: u8) -> CardSet {
        duel.get_matching_group(|c, duel| matfilter(c, duel, tp), tp, LOCATION_MZONE, LOCATION_MZONE, None)
    }

    fn op(duel: &mut Duel, g: CardSet, _: u8, _: CardId) -> Script<()> {
        duel.send_to_grave(g, REASON_COST | REASON_MATERIAL).map(|_| ())
    }
}

// In the Extra Deck, summoned by contact with Fallen of Albaz and Blue-Eyes White Dragon on the field, and in the GY after the
// opponent's Dark Hole, with Polymerization in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (CARD_ALBAZ, MZONE),
    (89631139, MZONE),
    (53129443, MZONE | OPPONENT),
    (CARD_POLYMERIZATION, DECK),
    (CARD_POLYMERIZATION, DECK),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_rep::<Materials>(duel, c);
    Fusion::add_contact_proc::<Contact>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_CANNOT_BE_FUSION_MATERIAL);
    e0.set_value(duel, 1);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_operation(duel, regop);
    c.register_effect(duel, e2);
}

fn splimit(_: &Duel, _: EffectId, _: Option<EffectId>, _: u8, st: u32, _: u8, _: u8) -> bool {
    st & SUMMON_TYPE_FUSION == SUMMON_TYPE_FUSION
}

fn matfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_able_to_grave_as_cost(duel) && (c.is_faceup(duel) || c.is_controler(duel, tp))
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_EXTRA_ATTACK_MONSTER);
    e1.set_value(duel, c.get_material_count(duel) as i32 - 1);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
    c.register_effect(duel, e1);
    Ok(())
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_PHASE + PHASE_END);
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    c.register_effect(duel, e1);
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_normal_spell(duel) && c.is_set_card(duel, &[SET_FUSION]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
