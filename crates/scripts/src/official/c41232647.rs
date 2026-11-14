// House Dragonmaid
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 41232647;

pub const LISTED_SERIES: &[u16] = &[SET_DRAGONMAID];

const KITCHEN_DRAGONMAID: u32 = 16960120;
const DRAGONMAID_CEHRMBA: u32 = 12163590;
const LADYS_DRAGONMAID: u32 = 48658295;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_DRAGONMAID], Some(fc), sumtype, tp)),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_race_full(duel, RACE_DRAGON, Some(fc), sumtype, tp)),
    ];
}

// In the Extra Deck, Fusion Summoned with Polymerization from Dragonmaids, and on the field with Lady's Dragonmaid in the
// Graveyard (one Level from Cehrmba) to Special Summon, and Kitchen Dragonmaid and Cehrmba to return to the hand.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (KITCHEN_DRAGONMAID, HAND),
    (KITCHEN_DRAGONMAID, MZONE),
    (KITCHEN_DRAGONMAID, GRAVE),
    (DRAGONMAID_CEHRMBA, HAND),
    (DRAGONMAID_CEHRMBA, MZONE),
    (DRAGONMAID_CEHRMBA, GRAVE),
    (LADYS_DRAGONMAID, GRAVE),
    (LADYS_DRAGONMAID, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(0),
    Check::SpecialSummoned,
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_TO_HAND);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, descon);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_set_card(duel, &[SET_DRAGONMAID])
        && c.is_level_above(duel, 1)
        && duel.is_existing_matching_card(
            |sc, duel| spfilter(sc, duel, e, tp, c),
            tp,
            LOCATION_HAND | LOCATION_GRAVE,
            0,
            1,
            None,
        )
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, tc: CardId) -> bool {
    c.is_set_card(duel, &[SET_DRAGONMAID])
        && c.is_level_above(duel, 1)
        && c.get_level(duel).abs_diff(tc.get_level(duel)) == 1
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let target_filter = |tc, duel: &Duel| filter(tc, duel, e, tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && target_filter(chkc, duel) && chkc != c);
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(target_filter, tp, LOCATION_MZONE, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, target_filter, tp, LOCATION_MZONE, 0, 1, 1, Some(c.into()))?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_GRAVE | LOCATION_HAND) as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if duel.get_location_count(tp, LOCATION_MZONE) > 0 && tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sp_filter = |c, duel: &Duel| spfilter(c, duel, e, tp, tc);
        let g = duel.select_matching_card(
            tp,
            aux::necro_valley_filter(sp_filter),
            tp,
            LOCATION_GRAVE | LOCATION_HAND,
            0,
            1,
            1,
            None,
        )?;
        if !g.is_empty() {
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
        }
    }
    Ok(())
}

fn descfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.get_previous_race_on_field(duel) & RACE_DRAGON != 0
        && c.get_previous_controler(duel) == tp
        && c.is_previous_location(duel, LOCATION_MZONE)
        && c.is_previous_position(duel, POS_FACEUP)
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.unwrap_or_default()
        .is_exists(duel, |c, duel| descfilter(c, duel, tp), 1, Some(a.e.get_handler(duel).into()))
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) {
        duel.destroy(tc, REASON_EFFECT)?;
    }
    Ok(())
}
