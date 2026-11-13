// Astrograph Sorcerer, the Star Magician
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT, PZONE};
use engine::{FusionCheck, FusionMaterialFn};

const ID: u32 = 48171151;
// s.desgroup, the monsters destroyed in the current chain, kept as a script group.
const DESGROUP: u64 = (ID as u64) << 16;

struct Params;

impl FusionSummonParams for Params {
    const EXTRAOP: Option<FusionMaterialFn> = Some(Fusion::shuffle_material);

    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_race(duel, RACE_DRAGON)
    }

    fn matfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        Fusion::on_field_mat(c, duel)
    }

    fn extrafil(duel: &Duel, _: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        let filter = aux::faceup_filter(CardId::is_able_to_deck);
        (Some(duel.get_matching_group(filter, tp, LOCATION_EXTRA, 0, None)), None)
    }
}

pub const LISTED_NAMES: &[u32] = &[ID];
// In the Pendulum Zone with Stargazer Magician (Scale 1) in the hand and Deck, in the hand with the opponent's Dark Hole to
// destroy monsters, and on the field with a Dragon Fusion Monster (Odd-Eyes Vortex Dragon) in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (ID, HAND),
    (ID, MZONE),
    (94415058, HAND),
    (94415058, DECK),
    (94415058, MZONE),
    (94415058, GRAVE),
    (53262004, EXTRA),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, pltg);
    e1.set_operation(duel, plop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP);
    e2.set_code(duel, EVENT_CUSTOM + ID);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    aux::global_check(duel, ID, |duel| {
        duel.set_script_group(DESGROUP, Some(CardSet::default()));
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_DESTROYED);
        ge1.set_operation(duel, desgroupregop);
        duel.register_effect(ge1, 0);
    });
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(
        duel,
        CATEGORY_RELEASE + CATEGORY_TODECK + CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON,
    );
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_condition(duel, |duel, _| duel.is_main_phase_full(None));
    e3.set_target(duel, tribtg);
    e3.set_operation(duel, tribop);
    e3.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    c.register_effect(duel, e3);
}

fn plfilter(c: CardId, duel: &Duel) -> bool {
    c.is_pendulum_monster(duel) && c.is_scale(duel, &[1]) && !c.is_code(duel, ID) && !c.is_forbidden(duel)
}

fn pltg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(plfilter, tp, LOCATION_HAND | LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(a.e.get_handler(duel).into()), 1, tp, 0);
    Ok(false)
}

fn plop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && duel.destroy(c, REASON_EFFECT)? > 0 && duel.check_pendulum_zones(tp) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
        let g = duel.select_matching_card(tp, plfilter, tp, LOCATION_HAND | LOCATION_DECK, 0, 1, 1, None)?;
        if let Some(sc) = g.get_first()
            && duel.move_to_field(sc, tp, tp, LOCATION_PZONE, POS_FACEUP, true)?
        {
            let e1 = duel.create_effect(c);
            e1.set_description(duel, 3302);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
            e1.set_code(duel, EFFECT_CANNOT_TRIGGER);
            e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
            sc.register_effect(duel, e1);
        }
    }
    Ok(())
}

fn spconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster_card(duel)
        && c.is_previous_position(duel, POS_FACEUP)
        && c.is_previous_location(duel, LOCATION_ONFIELD)
        && c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
}

fn desgroupregop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tg = a.eg.unwrap_or_default().filter(duel, spconfilter, None);
    if tg.is_empty() {
        return Ok(());
    }
    for tc in tg {
        tc.register_flag_effect(duel, ID, RESET_CHAIN, 0, 1);
    }
    let mut desgroup = duel.get_script_group(DESGROUP).unwrap_or_default();
    if duel.get_current_chain() == 0 {
        desgroup.clear();
    }
    desgroup.merge(tg);
    desgroup.remove_matching(duel, |c, duel| !c.has_flag_effect(duel, ID), None);
    duel.set_script_group(DESGROUP, Some(desgroup));
    for player in 0..2 {
        if tg.is_exists(duel, |c, duel| c.is_previous_controler(duel, player), 1, None) {
            let g = desgroup.filter(duel, |c, duel| c.is_previous_controler(duel, player), None);
            duel.raise_event(g, EVENT_CUSTOM + ID, a.re, a.r, a.rp, player, a.ev)?;
        }
    }
    Ok(())
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(a.ep == tp
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    let g = duel.get_script_group(DESGROUP).unwrap_or_default().filter(duel, spconfilter, None);
    e.set_label_object_group(duel, Some(g));
    for ec in g {
        ec.create_effect_relation(duel, e);
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(g), 1, tp, 0);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mmz_chk: bool) -> bool {
    if !(c.is_controler(duel, tp) && c.is_relate_to_effect(duel, e) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)) {
        return false;
    }
    if c.is_location(duel, LOCATION_GRAVE) {
        mmz_chk
    } else if c.is_location(duel, LOCATION_EXTRA) {
        duel.get_location_count_from_ex(tp, tp, None, c) > 0
    } else {
        false
    }
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        let mmz_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
        let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp, mmz_chk));
        let g = e.get_label_object_group(duel).unwrap_or_default().filter(duel, filter, None);
        if g.is_empty() || !duel.select_yes_no(tp, aux::stringid(ID, 3))? {
            return Ok(());
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = g.select(duel, tp, 1, 1, None)?;
        if !sg.is_empty() {
            duel.break_effect()?;
            duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}

fn tribtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(c.is_releasable_by_effect(duel));
    }
    duel.set_operation_info(0, CATEGORY_RELEASE, Some(c.into()), 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_TODECK, None, 1, tp, (LOCATION_ONFIELD | LOCATION_EXTRA) as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn tribop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && duel.release(c, REASON_EFFECT)? > 0 {
        Fusion::new_summon_eff_closure(duel, e, false);
        if Fusion::summon_eff_tg::<Params>(duel, a, 0, None)? && duel.select_yes_no(tp, aux::stringid(ID, 4))? {
            duel.break_effect()?;
            Fusion::new_summon_eff_closure(duel, e, true);
            Fusion::summon_eff_op::<Params>(duel, a)?;
        }
    }
    Ok(())
}
