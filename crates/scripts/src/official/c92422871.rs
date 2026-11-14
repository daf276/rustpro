// Cyberse Desavewurm
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 92422871;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(matfilter),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_race_full(duel, RACE_CYBERSE, Some(fc), sumtype, tp)),
    ];
}

// On the field with Cyberse Wizards in hand to summon, The Arrival Cyberse @Ignister (Link 6) to negate the opponent's
// Raigeki, and Cyberse Quantum Dragon to Fusion Summon a copy with.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (36033786, HAND),
    (36033786, HAND),
    (11738489, MZONE),
    (63533837, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (12580477, HAND | OPPONENT),
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
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_LVCHANGE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_NEGATE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE | LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, negcon);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, negtg);
    e2.set_operation(duel, negop);
    c.register_effect(duel, e2);
    aux::double_snare_validity(duel, c, LOCATION_MZONE | LOCATION_GRAVE);
}

fn matfilter(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_type_full(
        duel,
        TYPE_RITUAL | TYPE_FUSION | TYPE_SYNCHRO | TYPE_XYZ | TYPE_LINK,
        Some(fc),
        sumtype,
        tp,
    ) && c.is_race_full(duel, RACE_CYBERSE, Some(fc), sumtype, tp)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_CYBERSE) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, e, tp), LOCATION_HAND)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) == 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let tc = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
    let c = e.get_handler(duel);
    if let Some(tc) = tc.get_first()
        && duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)? > 0
        && c.is_relate_to_effect(duel, e)
        && c.is_faceup(duel)
        && c.has_level(duel)
        && !c.is_level(duel, &[tc.get_level(duel)])
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.break_effect()?;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CHANGE_LEVEL);
        e1.set_value(duel, tc.get_level(duel) as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_CYBERSE) && c.is_link_above(duel, 4)
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED)
        && a.ep == 1 - a.tp
        && a.re.is_some_and(|re| re.is_spell_trap_effect(duel))
        && duel.is_chain_negatable(a.ev as u8)
        && duel.is_existing_matching_card(cfilter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.negate_activation(a.ev as u8);
    Ok(())
}
