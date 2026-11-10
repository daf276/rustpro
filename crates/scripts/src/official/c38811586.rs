// Albion the Sanctifire Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 38811586;
pub const LISTED_NAMES: &[u32] = &[CARD_ALBAZ];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(CARD_ALBAZ), FusionMaterial::Filter(matfilter)];
}

// On the field and in the Extra Deck, Fusion Summoned by Polymerization from Fallen of Albaz and Mystical Elf, with Sangan
// in both Graveyards to Special Summon on the opponent's turn.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (CARD_ALBAZ, HAND),
    (15025844, HAND),
    (26202165, GRAVE),
    (26202165, GRAVE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_CANNOT_BE_FUSION_MATERIAL);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e3.set_count_limit(duel, 1, (ID, 0));
    e3.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e3.set_target(duel, sptg1);
    e3.set_operation(duel, spop1);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_RELEASE);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_GRAVE);
    e4.set_count_limit(duel, 1, (ID, 1));
    e4.set_target(duel, sptg2);
    e4.set_operation(duel, spop2);
    c.register_effect(duel, e4);
}

fn matfilter(c: CardId, duel: &Duel, scard: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_attribute_full(duel, ATTRIBUTE_LIGHT, Some(scard), sumtype, tp) && c.is_race_full(duel, RACE_SPELLCASTER, Some(scard), sumtype, tp)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, targetp: u8) -> bool {
    c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, targetp, 0xff) && c.is_can_be_effect_target(duel, e)
}

fn sptg1(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let g1 = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp, tp), tp, LOCATION_GRAVE, LOCATION_GRAVE, None);
    let g2 = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp, 1 - tp), tp, LOCATION_GRAVE, LOCATION_GRAVE, None);
    if chk == 0 {
        let both = g1.intersection(g2);
        return Ok(
            ((g1.len() > 1 && g2.len() > 1) || (both.len() != g1.len() && both.len() != g2.len()))
                && duel.get_location_count(tp, LOCATION_MZONE) > 0
                && duel.get_location_count_full(1 - tp, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, 0xff) > 0
                && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none(),
        );
    }
    let rescon =
        |_: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>| (!sg.intersection(g1).is_empty() && !sg.intersection(g2).is_empty(), false);
    let tg = aux::select_unselect_group(duel, g1.union(g2), 2, 2, &rescon, tp, HINTMSG_SPSUMMON)?;
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(tg), 2, 0, 0);
    Ok(false)
}

fn spownfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, tg: CardSet) -> bool {
    c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && tg
            .difference(c.into())
            .first()
            .is_some_and(|tc| tc.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, 1 - tp, 0xff))
}

fn spop1(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tg = duel.get_target_cards(e).unwrap_or_default();
    if tg.len() < 2
        || duel.get_location_count(tp, LOCATION_MZONE) < 1
        || duel.get_location_count_full(1 - tp, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, 0xff) < 1
        || duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some()
    {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
    let Some(sc) = tg
        .filter_select(duel, tp, |c, duel| spownfilter(c, duel, e, tp, tg), 1, 1, None)?
        .first()
    else {
        return Ok(());
    };
    duel.special_summon_step(sc, 0, tp, tp, false, false, POS_FACEUP)?;
    if let Some(other) = tg.difference(sc.into()).first() {
        duel.special_summon_step(other, 0, tp, 1 - tp, false, false, POS_FACEUP)?;
    }
    duel.special_summon_complete()?;
    Ok(())
}

fn spcfilter(c: CardId, duel: &Duel) -> bool {
    c.is_releasable_by_effect(duel) && (c.is_sequence(duel, 2) || c.is_in_extra_mzone(duel))
}

fn sptg2(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let g = duel.get_matching_group(spcfilter, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    if chk == 0 {
        return Ok(g.len() == 4 && duel.get_mzone_count(tp, Some(g)) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_RELEASE, Some(g), g.len() as u8, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn spop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let g = duel.get_matching_group(spcfilter, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    if g.len() == 4 && duel.release(g, REASON_EFFECT)? == 4 && c.is_relate_to_effect(duel, e) {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
