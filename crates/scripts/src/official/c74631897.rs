// Artmage Non-Finito
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 74631897;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_ARTMAGE], Some(fc), sumtype, tp)),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_ARTMAGE], Some(fc), sumtype, tp)),
    ];
}

struct Params;

impl FusionSummonParams for Params {
    fn matfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_on_field(duel) && c.is_able_to_grave(duel)
    }
}

pub const LISTED_NAMES: &[u32] = &[ID];

// In the Extra Deck, Fusion Summoned by Polymerization with two Medius the Pure, or Special Summoned by discarding a Spell and
// Tributing Artmage Diactorus. No Normal Monster or ported card is an "Artmage" Spell/Trap, so the Set trigger never resolves.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (24094653, HAND),
    (CARD_MEDIUS_THE_PURE, HAND),
    (CARD_MEDIUS_THE_PURE, HAND),
    (27184601, MZONE),
    (12580477, HAND),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 2))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
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
    e2.set_category(duel, CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_target(duel, settg);
    e2.set_operation(duel, setop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e3.set_target(duel, Fusion::summon_eff_tg::<Params>);
    e3.set_operation(duel, Fusion::summon_eff_op::<Params>);
    e3.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e3);
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
        c.is_code(duel, ID)
            && (sumtype & SUMMON_TYPE_FUSION == SUMMON_TYPE_FUSION || sumtype & (SUMMON_TYPE_SPECIAL + 1) == SUMMON_TYPE_SPECIAL + 1)
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, a.tp);
    Ok(())
}

fn selfspcostfilter(c: CardId, duel: &Duel, tp: u8, fc: CardId) -> bool {
    c.is_level_above(duel, 7)
        && c.is_set_card_full(duel, &[SET_ARTMAGE], Some(fc), (SUMMON_TYPE_SPECIAL + 1).into(), tp)
        && duel.get_location_count_from_ex(tp, tp, Some(c.into()), fc) > 0
}

fn discardfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel) && c.is_discardable(duel)
}

fn selfspcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    if c.is_faceup(duel) {
        return false;
    }
    let tp = c.get_controler(duel);
    let options = ReleaseOptions {
        use_hand: true,
        check_field: true,
        to_check: Some(c),
        ..ReleaseOptions::new(tp)
    };
    duel.is_existing_matching_card(discardfilter, tp, LOCATION_HAND, 0, 1, None)
        && duel.check_release_group_full(tp, |rc, duel| selfspcostfilter(rc, duel, tp, c), 1, options)
}

fn selfsptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let Some(c) = a.c else { return Ok(false) };
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DISCARD);
    let dg = duel.select_matching_card_full(tp, discardfilter, tp, LOCATION_HAND, 0, 1, 1, true, None)?;
    if dg.is_empty() {
        return Ok(false);
    }
    let options = ReleaseOptions {
        use_hand: true,
        check_field: true,
        to_check: Some(c),
        ..ReleaseOptions::new(tp)
    };
    let rg = duel.select_release_group_full(tp, |rc, duel| selfspcostfilter(rc, duel, tp, c), (1, 1), true, options)?;
    if rg.is_empty() {
        return Ok(false);
    }
    e.set_label_object_group(duel, Some(dg.union(rg)));
    Ok(true)
}

fn selfspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(g) = a.e.get_label_object_group(duel) else { return Ok(()) };
    if g.len() != 2 {
        return Ok(());
    }
    let (dg, rg) = g.split(duel, |c, duel| c.is_spell_trap(duel), None);
    duel.send_to_grave(dg, REASON_COST | REASON_MATERIAL | REASON_DISCARD)?;
    duel.release(rg, REASON_COST | REASON_MATERIAL)?;
    Ok(())
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ARTMAGE]) && c.is_spell_trap(duel) && c.is_ssetable(duel)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && duel.is_existing_matching_card(setfilter, a.tp, LOCATION_DECK, 0, 1, None))
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let g = duel.select_matching_card(tp, setfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.sset(tp, g)?;
    }
    Ok(())
}
