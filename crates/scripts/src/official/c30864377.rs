// Gladiator Beast Tamer Editor
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE, OPPONENT};

const ID: u32 = 30864377;

pub const LISTED_SERIES: &[u16] = &[SET_GLADIATOR];
pub const LISTED_NAMES: &[u32] = &[ID];
pub const MATERIAL_SETCODE: &[u16] = &[SET_GLADIATOR];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Filter(matfilter), FusionMaterial::Filter(matfilter)];
}

// Contact Summoned from Gladiator Beast Augustus on the field, with Heraklinos in the Extra Deck, Hoplomus in the Deck and
// monsters of the opponent's to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (7573135, MZONE),
    (7573135, MZONE),
    (7573135, MZONE),
    (27346636, EXTRA),
    (27346636, EXTRA),
    (4253484, DECK),
    (4253484, DECK),
    (27125110, MZONE | OPPONENT),
    (27125110, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 3)),
    Check::ResolvesEffect(aux::stringid(ID, 4)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    Fusion::add_contact_proc::<super::c27346636::Contact>(duel, c);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_CANNOT_BE_FUSION_MATERIAL);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e3.set_value(duel, 1);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 3));
    e4.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, 0);
    e4.set_target(duel, esptg);
    e4.set_operation(duel, espop);
    c.register_effect(duel, e4);
    let e6 = duel.create_effect(c);
    e6.set_description(duel, aux::stringid(ID, 4));
    e6.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e6.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e6.set_code(duel, EVENT_PHASE | PHASE_BATTLE);
    e6.set_range(duel, LOCATION_MZONE);
    e6.set_count_limit(duel, 1, 0);
    e6.set_cost(duel, spcost);
    e6.set_target(duel, sptg);
    e6.set_operation(duel, spop);
    c.register_effect(duel, e6);
}

fn matfilter(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_level_above(duel, 5) && c.is_set_card_full(duel, &[SET_GLADIATOR], Some(fc), sumtype, tp)
}

fn espfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_GLADIATOR])
        && c.is_type(duel, TYPE_FUSION)
        && !c.is_code(duel, ID)
        && duel.get_location_count_from_ex(tp, tp, None, c) > 0
        && c.is_can_be_special_summoned(duel, e, 124, tp, true, false)
}

fn esptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    stock::exists_target(
        duel,
        a,
        chk,
        |c, duel| espfilter(c, duel, e, tp),
        LOCATION_EXTRA,
        CATEGORY_SPECIAL_SUMMON,
    )
}

fn espop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| espfilter(c, duel, e, tp), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 124, tp, tp, true, false, POS_FACEUP)?;
    }
    Ok(())
}

fn spcfilter(c: CardId, duel: &Duel, ft: i32) -> bool {
    c.is_faceup(duel)
        && c.is_set_card(duel, &[SET_GLADIATOR])
        && c.get_battled_group_count(duel) > 0
        && c.is_able_to_deck_or_extra_as_cost(duel)
        && (ft > 0 || c.get_sequence(duel) < 5)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    if chk == 0 {
        return Ok(ft > -1 && duel.is_existing_matching_card(|c, duel| spcfilter(c, duel, ft), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_matching_card(tp, |c, duel| spcfilter(c, duel, ft), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.confirm_cards(1 - tp, g)?;
    duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_COST)?;
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_GLADIATOR]) && c.is_can_be_special_summoned(duel, e, 124, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    stock::exists_target(
        duel,
        a,
        chk,
        |c, duel| spfilter(c, duel, e, tp),
        LOCATION_DECK,
        CATEGORY_SPECIAL_SUMMON,
    )
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        duel.special_summon_step(tc, 124, tp, tp, false, false, POS_FACEUP)?;
        let code = tc.get_original_code(duel);
        tc.register_flag_effect(duel, code, RESET_EVENT | RESETS_STANDARD_DISABLE, 0, 0);
        duel.special_summon_complete()?;
    }
    Ok(())
}
