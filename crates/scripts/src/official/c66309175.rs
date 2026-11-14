// Predaplant Ambulomelides
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 66309175;

pub const COUNTER_LIST: &[u16] = &[COUNTER_PREDATOR];
pub const LISTED_SERIES: &[u16] = &[SET_PREDAP, SET_PREDAPLANT];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = false;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] =
        &[FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_PREDAPLANT], Some(fc), sumtype, tp)); 2];
}

// Fusion Summoned by Polymerization with Predaplant Byblisp and Lilizard, which are also in the Deck to search and summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (44932065, MZONE),
    (18595008, HAND),
    (18595008, MZONE),
    (44932065, DECK),
    (18595008, DECK),
    (89176044, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_fusion_summoned(duel));
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_RELEASE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_hand(duel)
        && (c.is_faceup(duel) || !c.is_location(duel, LOCATION_EXTRA))
        && ((c.is_set_card(duel, &[SET_PREDAP]) && c.is_spell_trap(duel)) || (c.is_set_card(duel, &[SET_PREDAPLANT]) && c.is_monster(duel)))
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(
        duel,
        a,
        chk,
        thfilter,
        LOCATION_DECK | LOCATION_GRAVE | LOCATION_EXTRA,
        CATEGORY_TOHAND,
    )
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(
        duel,
        a,
        aux::necro_valley_filter(thfilter),
        LOCATION_DECK | LOCATION_GRAVE | LOCATION_EXTRA,
    )
}

fn spcfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    (c.is_controler(duel, tp) || (c.is_faceup(duel) && c.get_counter(duel, COUNTER_PREDATOR) > 0))
        && c.is_releasable_by_effect(duel)
        && duel.get_mzone_count(tp, Some(c.into())) > 0
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_PREDAPLANT]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && spcfilter(chkc, duel, tp));
    }
    if chk == 0 {
        return Ok(
            duel.is_existing_target(|c, duel| spcfilter(c, duel, tp), tp, LOCATION_MZONE, LOCATION_MZONE, 1, None)
                && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None),
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RELEASE);
    duel.select_target(tp, |c, duel| spcfilter(c, duel, tp), tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e)
        && !tc.is_immune_to_effect(duel, e)
        && duel.release(tc, REASON_EFFECT)? > 0
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
