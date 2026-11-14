// Favorite HERO Flame Wingman
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 13243124;

pub const LISTED_SERIES: &[u16] = &[SET_ELEMENTAL_HERO];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::GroupFilter(matfilter); 2];
}

struct Params;

impl FusionSummonParams for Params {}

// In the Extra Deck, Fusion Summoned by Polymerization with Elemental HERO Avian and Burstinatrix (Warriors of different Attributes),
// attacked by Blue-Eyes White Dragon (3000 ATK) with a Level 2 Normal Monster in the hand, and a Fusion Monster to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (24094653, HAND),
    (24094653, HAND),
    (21844576, HAND),
    (21844576, MZONE),
    (58932615, HAND),
    (58932615, MZONE),
    (756652, HAND),
    (89631139, MZONE | OPPONENT),
    (89631139, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, atkspcon);
    e1.set_target(duel, atksptg);
    e1.set_operation(duel, atkspop);
    e1.set_count_limit(duel, 1, (ID, 0));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, _| duel.get_flag_effect(0, ID) > 0);
    e2.set_target(duel, Fusion::summon_eff_tg::<Params>);
    e2.set_operation(duel, Fusion::summon_eff_op::<Params>);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_hint_timing(
        duel,
        0,
        TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMING_BATTLE_END | TIMINGS_CHECK_MONSTER_E,
    );
    c.register_effect(duel, e2);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_BATTLE_DESTROYED);
        ge1.set_operation(duel, |duel, _| {
            duel.register_flag_effect(0, ID, RESET_PHASE | PHASE_END, 0, 1);
            Ok(())
        });
        duel.register_effect(ge1, 0);
    });
}

fn matfilter(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8, sg: Option<CardSet>) -> bool {
    let Some(sg) = sg else { return true };
    sg.filter_count(duel, aux::TRUE, Some(c.into())) == 0
        || (sg.is_exists(
            duel,
            |mc, duel| mc.is_race_full(duel, c.get_race(duel), Some(fc), sumtype, tp),
            1,
            Some(c.into()),
        ) && !sg.is_exists(
            duel,
            |mc, duel| mc.is_attribute_full(duel, c.get_attribute(duel), Some(fc), sumtype, tp),
            1,
            Some(c.into()),
        ))
}

fn atkspcon(duel: &Duel, a: &Args) -> bool {
    let (bc0, bc1) = duel.get_battle_monster(a.tp);
    matches!((bc0, bc1), (Some(bc0), Some(bc1)) if bc0 == a.e.get_handler(duel) && bc1.is_attack_above(duel, 2200) && bc1.is_faceup(duel))
}

fn atksptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_ATKCHANGE, Some(a.e.get_handler(duel).into()), 1, tp, 1000);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_below(duel, 4) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn atkspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e)
        && c.is_faceup(duel)
        && c.update_attack(duel, 1000, None, None, None) > 0
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(|sc, duel| spfilter(sc, duel, e, tp), tp, LOCATION_HAND, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |sc, duel| spfilter(sc, duel, e, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.break_effect()?;
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
        }
    }
    Ok(())
}
