// Lady's Dragonmaid
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, MZONE};

const ID: u32 = 48658295;

pub const LISTED_SERIES: &[u16] = &[SET_DRAGONMAID];

const KITCHEN_DRAGONMAID: u32 = 16960120;
const HOUSE_DRAGONMAID: u32 = 41232647;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::GroupFilter(matfilter); 2];
}

struct Contact;

impl ContactFusion for Contact {
    const SUMCON: Option<Value> = Some(Value::SpSummonCondition(splimit));

    fn group(duel: &Duel, tp: u8) -> CardSet {
        // The Lua returns false here, which errors in CheckFusionMaterial and SelectFusionMaterial.
        unported!(duel, duel.is_player_affected_by_effect(tp, CARD_SPIRIT_ELIMINATION).is_some());
        duel.get_matching_group(CardId::is_able_to_remove_as_cost, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, None)
    }

    fn op(duel: &mut Duel, g: CardSet, _: u8, _: CardId) -> Script<()> {
        duel.remove(g, POS_FACEUP, REASON_COST | REASON_MATERIAL)?;
        Ok(())
    }
}

struct Params;

impl FusionSummonParams for Params {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_race(duel, RACE_DRAGON)
    }

    fn matfilter(_: CardId, _: &Duel, _: EffectId, _: u8) -> bool {
        false
    }

    fn extrafil(duel: &Duel, _: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        let filter = |c, duel: &Duel| Fusion::is_monster_filter(c, duel, fextrafil);
        (
            Some(duel.get_matching_group(filter, tp, LOCATION_ONFIELD | LOCATION_REMOVED, 0, None)),
            None,
        )
    }

    const EXTRAOP: Option<FusionMaterialFn> = Some(Fusion::shuffle_material);

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk != 0 {
            duel.set_operation_info(0, CATEGORY_TODECK, None, 0, a.tp, (LOCATION_ONFIELD | LOCATION_REMOVED) as i32);
        }
        Ok(true)
    }
}

// In the Extra Deck, contact Summoned from LIGHT Level 8 and 9 Fusion Dragonmaids on the field, with Kitchen
// Dragonmaid in the Deck to Special Summon and House Dragonmaid to fuse into.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (HOUSE_DRAGONMAID, MZONE),
    (HOUSE_DRAGONMAID, GRAVE),
    (HOUSE_DRAGONMAID, EXTRA),
    (KITCHEN_DRAGONMAID, DECK),
    (KITCHEN_DRAGONMAID, DECK),
    (KITCHEN_DRAGONMAID, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    Fusion::add_contact_proc::<Contact>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, Fusion::summon_eff_tg::<Params>);
    e2.set_operation(duel, Fusion::summon_eff_op::<Params>);
    c.register_effect(duel, e2);
}

fn matfilter(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8, sg: Option<CardSet>) -> bool {
    let others = |sg: CardSet| sg.is_exists(duel, aux::TRUE, 1, Some(c.into()));
    c.is_set_card_full(duel, &[SET_DRAGONMAID], Some(fc), sumtype, tp)
        && sg.is_none_or(|sg| {
            !others(sg)
                || (sg.is_exists(
                    duel,
                    |mc, duel| mc.is_attribute_full(duel, c.get_attribute(duel), Some(fc), sumtype, tp),
                    1,
                    Some(c.into()),
                ) && !sg.is_exists(duel, |mc, duel| mc.is_level(duel, &[c.get_level(duel)]), 1, Some(c.into()))
                    && !sg.is_exists(duel, |mc, duel| mc.is_location(duel, c.get_location(duel)), 1, Some(c.into())))
        })
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_below(duel, 4) && c.is_set_card(duel, &[SET_DRAGONMAID]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let filter = |c, duel: &Duel| spfilter(c, duel, e, tp);
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_matching_card(filter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = |c, duel: &Duel| spfilter(c, duel, e, tp);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn fextrafil(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_deck(duel) && (c.is_on_field(duel) || c.is_faceup(duel))
}

fn splimit(duel: &Duel, e: EffectId, _: Option<EffectId>, _: u8, _: u32, _: u8, _: u8) -> bool {
    !e.get_handler(duel).is_location(duel, LOCATION_EXTRA)
}
