// Dark Magician of Destruction
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 59400890;
pub const MATERIAL_SETCODE: &[u16] = &[SET_DARK_MAGICIAN];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Code(CARD_DARK_MAGICIAN),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| {
            c.is_attribute_full(duel, ATTRIBUTE_LIGHT | ATTRIBUTE_DARK, Some(fc), sumtype, tp)
        }),
    ];
}

pub const LISTED_NAMES: &[u32] = &[CARD_DARK_MAGICIAN];

// Fusion Summoned with Polymerization, Dark Magician and Sangan, another Dark Magician to search; or with Magician of Black Chaos as the banished cost.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (24094653, HAND),
    (CARD_DARK_MAGICIAN, HAND),
    (CARD_DARK_MAGICIAN, DECK),
    (26202165, HAND),
    (30208479, MZONE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    c.add_must_be_fusion_summoned(duel);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e0.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e0.set_condition(duel, regcon);
    e0.set_operation(duel, regop);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_range(duel, LOCATION_EXTRA);
    e1.set_condition(duel, selfspcon);
    e1.set_target(duel, selfsptg);
    e1.set_operation(duel, selfspop);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_code(duel, EFFECT_CHANGE_CODE);
    e3.set_range(duel, LOCATION_MZONE | LOCATION_GRAVE);
    e3.set_value(duel, CARD_DARK_MAGICIAN as i32);
    c.register_effect(duel, e3);
    duel.add_custom_activity_counter(ID, ACTIVITY_CHAIN, CounterFilter::Chain(|duel, re, _, _| !re.is_spell_effect(duel)));
}

fn regcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_fusion_summoned(duel) || c.is_summon_type(duel, &[SUMMON_TYPE_SPECIAL + 1])
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, sumtype, _, _, _| {
        c.is_original_code(duel, ID)
            && (sumtype & SUMMON_TYPE_FUSION == SUMMON_TYPE_FUSION || sumtype & (SUMMON_TYPE_SPECIAL + 1) == SUMMON_TYPE_SPECIAL + 1)
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, a.tp);
    Ok(())
}

fn selfspcostfilter(c: CardId, duel: &Duel, tp: u8, sc: CardId) -> bool {
    c.is_level_above(duel, 6)
        && c.is_attribute(duel, ATTRIBUTE_DARK)
        && c.is_race(duel, RACE_SPELLCASTER)
        && c.is_able_to_remove_as_cost(duel)
        && duel.get_location_count_from_ex(tp, tp, Some(c.into()), sc) > 0
}

fn selfspcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    (duel.get_custom_activity_count(ID, tp, ACTIVITY_CHAIN) > 0 || duel.get_custom_activity_count(ID, 1 - tp, ACTIVITY_CHAIN) > 0)
        && duel.is_existing_matching_card(|mc, duel| selfspcostfilter(mc, duel, tp, c), tp, LOCATION_MZONE, 0, 1, None)
}

fn selfsptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let Some(c) = a.c else { return Ok(false) };
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card_full(
        tp,
        |mc, duel| selfspcostfilter(mc, duel, tp, c),
        tp,
        LOCATION_MZONE,
        0,
        1,
        1,
        true,
        None,
    )?;
    if g.is_empty() {
        return Ok(false);
    }
    e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn selfspop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(g) = a.e.get_label_object_group(duel) {
        duel.remove(g, POS_FACEUP, REASON_COST | REASON_MATERIAL)?;
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_code(duel, CARD_DARK_MAGICIAN) || c.lists_code(duel, CARD_DARK_MAGICIAN)) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
