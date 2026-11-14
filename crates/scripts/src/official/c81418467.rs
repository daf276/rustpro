// Primite Imperial Dragon
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 81418467;

pub const LISTED_SERIES: &[u16] = &[SET_PRIMITE];
// In the hand with Mystical Elf (Normal Monster) on the field to Tribute and in the GY to match the opponent's monsters, and
// Primite Dragon Ether Beryl in the hand to Normal Summon on the opponent's turn.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (15025844, MZONE),
    (15025844, MZONE),
    (15025844, GRAVE),
    (63198739, HAND),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::TributeSummoned(1),
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_LIMIT_SUMMON_PROC);
    e0.set_condition(duel, selfnssumcon);
    e0.set_target(duel, selfnssumtg);
    e0.set_operation(duel, selfnssumop);
    e0.set_value(duel, SUMMON_TYPE_TRIBUTE as i32);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_MAIN_END);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp) && duel.is_main_phase());
    e1.set_cost(duel, Cost::self_reveal);
    e1.set_target(duel, sumtg);
    e1.set_operation(duel, sumop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DISABLE + CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_tribute_summoned(duel));
    e2.set_target(duel, efftg);
    e2.set_operation(duel, effop);
    c.register_effect(duel, e2);
}

fn tributefilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_type(duel, TYPE_NORMAL) && (c.is_controler(duel, tp) || c.is_faceup(duel))
}

fn rescon(tp: u8) -> impl Fn(&Duel, CardSet, CardSet, Option<CardId>) -> (bool, bool) {
    move |duel, sg, mg, c| {
        (
            aux::chkf_mmz(tp, 1)(duel, sg, mg, c).0 && sg.is_exists(duel, |c, duel| tributefilter(c, duel, tp), 1, None),
            false,
        )
    }
}

fn selfnssumcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    aux::select_unselect_group_check(duel, duel.get_release_group(tp), 1, 1, &rescon(tp))
}

fn selfnssumtg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let cancelable = duel.is_summon_cancelable();
    let rg = duel.get_release_group(tp);
    let g = aux::select_unselect_group_full(duel, rg, 1, 1, &rescon(tp), tp, HINTMSG_RELEASE, None, cancelable)?;
    if !g.is_empty() {
        a.e.set_label_object_group(duel, Some(g));
        return Ok(true);
    }
    Ok(false)
}

fn selfnssumop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (Some(c), Some(g)) = (a.c, a.e.get_label_object_group(duel)) else {
        return Ok(());
    };
    c.set_material(duel, Some(g));
    duel.release(g, REASON_SUMMON | REASON_COST | REASON_MATERIAL)?;
    Ok(())
}

fn sumfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_PRIMITE]) && c.is_summonable(duel, true, None)
}

fn sumtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(sumfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, None));
    }
    let g = duel.get_matching_group(sumfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, None);
    duel.set_operation_info(0, CATEGORY_SUMMON, Some(g), 1, tp, 0);
    Ok(false)
}

fn sumop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SUMMON);
    let tc = duel
        .select_matching_card(tp, sumfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, 1, None)?
        .get_first();
    if let Some(tc) = tc {
        duel.summon(tp, tc, true, None)?;
    }
    Ok(())
}

fn rmvfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    let (race, att) = (c.get_race(duel), c.get_attribute(duel));
    c.is_faceup(duel)
        && c.is_able_to_remove(duel)
        && duel.is_existing_matching_card(|c, duel| matchfilter(c, duel, race, att), tp, LOCATION_GRAVE, 0, 1, None)
}

fn matchfilter(c: CardId, duel: &Duel, race: u64, att: u32) -> bool {
    c.is_type(duel, TYPE_NORMAL) && (c.is_race(duel, race) || c.is_attribute(duel, att))
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let disg = duel.get_matching_group(CardId::is_negatable_monster, tp, 0, LOCATION_MZONE, None);
    let rmvg = duel.get_matching_group(|c, duel| rmvfilter(c, duel, tp), tp, 0, LOCATION_MZONE, None);
    if chk == 0 {
        return Ok(!disg.is_empty() || !rmvg.is_empty());
    }
    if !disg.is_empty() {
        duel.set_operation_info(0, CATEGORY_DISABLE, Some(disg), disg.len() as u8, tp, 0);
    }
    if !rmvg.is_empty() {
        duel.set_operation_info(0, CATEGORY_REMOVE, Some(rmvg), rmvg.len() as u8, tp, 0);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let mut break_chk = false;
    let disg = duel
        .get_matching_group(CardId::is_negatable_monster, tp, 0, LOCATION_MZONE, None)
        .filter(duel, |tc, duel| tc.is_can_be_disabled_by_effect(duel, e, true), None);
    for tc in disg {
        tc.negate_effects(duel, c, 0, false);
        duel.adjust_instantly(Some(tc));
        break_chk = true;
    }
    let rmvg = duel.get_matching_group(|c, duel| rmvfilter(c, duel, tp), tp, 0, LOCATION_MZONE, None);
    if !rmvg.is_empty() {
        if break_chk {
            duel.break_effect()?;
        }
        duel.remove(rmvg, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}
