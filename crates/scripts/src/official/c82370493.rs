// Predaplant Dheadevil
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 82370493;

pub const LISTED_SERIES: &[u16] = &[SET_PREDAPLANT];
pub const COUNTER_LIST: &[u16] = &[COUNTER_PREDATOR];

pub const MATERIAL_SETCODE: &[u16] = &[SET_PREDAPLANT];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_PREDAPLANT], Some(fc), sumtype, tp)),
        FusionMaterial::Filter(|c, duel, _, _, _| c.is_level(duel, &[1])),
    ];
}

// In the Extra Deck, with Dragostapelia putting a Predator Counter on Elfnote Lucina to Tribute for the Special Summon, and Spells/Traps to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (69946549, MZONE),
    (44932065, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (5318639, SZONE | OPPONENT),
    (5318639, SZONE | OPPONENT),
    (5318639, HAND),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    c.add_must_first_be_fusion_summoned(duel);
    let e0 = duel.create_effect(c);
    e0.set_description(duel, aux::stringid(ID, 0));
    e0.set_type(duel, EFFECT_TYPE_FIELD);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_SPSUMMON_PROC);
    e0.set_range(duel, LOCATION_EXTRA);
    e0.set_condition(duel, selfspcon);
    e0.set_target(duel, selfsptg);
    e0.set_operation(duel, selfspop);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| {
        duel.is_main_phase() && a.e.get_handler(duel).is_status(duel, STATUS_SPSUMMON_TURN)
    });
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    c.register_effect(duel, e1);
}

fn selfspcostfilter(c: CardId, duel: &Duel, tp: u8, fc: CardId) -> bool {
    ((c.is_set_card(duel, &[SET_PREDAPLANT]) && c.is_controler(duel, tp)) || c.has_counter(duel, COUNTER_PREDATOR))
        && c.is_releasable(duel)
        && c.is_can_be_fusion_material(duel, Some(fc), MATERIAL_FUSION)
}

fn predaplantchkfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_PREDAPLANT]) && c.is_controler(duel, tp)
}

fn rescon(duel: &Duel, sg: CardSet, e: EffectId, tp: u8) -> (bool, bool) {
    (
        duel.get_location_count_from_ex(tp, tp, Some(sg), e.get_handler(duel)) > 0
            && sg.is_exists(duel, |c, duel| predaplantchkfilter(c, duel, tp), 1, None)
            && sg.is_exists(duel, |c, duel| c.has_counter(duel, COUNTER_PREDATOR), 1, None),
        false,
    )
}

fn selfspcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    let g = duel.get_matching_group(
        |mc, duel| selfspcostfilter(mc, duel, tp, c),
        tp,
        LOCATION_MZONE,
        LOCATION_MZONE,
        None,
    );
    aux::select_unselect_group_check(duel, g, 2, 2, &|duel, sg, _, _| rescon(duel, sg, a.e, tp))
}

fn selfsptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let Some(c) = a.c else { return Ok(false) };
    let g = duel.get_matching_group(
        |mc, duel| selfspcostfilter(mc, duel, tp, c),
        tp,
        LOCATION_MZONE,
        LOCATION_MZONE,
        None,
    );
    let sg = aux::select_unselect_group_full(
        duel,
        g,
        2,
        2,
        &|duel, sg, _, _| rescon(duel, sg, e, tp),
        tp,
        HINTMSG_RELEASE,
        None,
        true,
    )?;
    if sg.len() == 2 {
        e.set_label_object_group(duel, Some(sg));
        return Ok(true);
    }
    Ok(false)
}

fn selfspop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(g) = a.e.get_label_object_group(duel) {
        duel.release(g, REASON_COST | REASON_MATERIAL)?;
    }
    Ok(())
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_spell_trap(duel));
    }
    let max_count = duel.get_matching_group_count(
        |c, duel| c.has_counter(duel, COUNTER_PREDATOR),
        tp,
        LOCATION_MZONE,
        LOCATION_MZONE,
        None,
    );
    if chk == 0 {
        return Ok(max_count > 0 && duel.is_existing_target(CardId::is_spell_trap, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(
        tp,
        CardId::is_spell_trap,
        tp,
        LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        1,
        max_count as u8,
        None,
    )?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, g.map_or(0, |g| g.len() as u8), tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if !tg.is_empty() {
        duel.destroy(tg, REASON_EFFECT)?;
    }
    Ok(())
}
