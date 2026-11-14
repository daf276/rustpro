// Berserkion the Electromagna Warrior
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 42901635;
const ALPHA: u32 = 42023223;
const BETA: u32 = 79418928;
const GAMMA: u32 = 15502037;

pub const LISTED_SERIES: &[u16] = &[SET_MAGNET_WARRIOR];
pub const LISTED_NAMES: &[u32] = &[ALPHA, BETA, GAMMA];

// Alpha, Beta and Gamma The Electromagnet Warrior to banish, Alpha The Magnet Warrior for the destroy cost, and the opponent's Dark Hole to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ALPHA, GRAVE),
    (ALPHA, HAND),
    (BETA, GRAVE),
    (BETA, HAND),
    (GAMMA, GRAVE),
    (GAMMA, HAND),
    (99785935, MZONE),
    (99785935, GRAVE),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_cost(duel, cost);
    e2.set_target(duel, target);
    e2.set_operation(duel, stock::destroy_first_target);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP);
    e3.set_condition(duel, spcon2);
    e3.set_target(duel, sptg2);
    e3.set_operation(duel, spop2);
    c.register_effect(duel, e3);
}

fn rescon(duel: &Duel, sg: CardSet, tp: u8) -> (bool, bool) {
    let cchk2 =
        |c: CardId, duel: &Duel| c.is_code(duel, BETA) && sg.filter_count(duel, |c, duel| c.is_code(duel, GAMMA), Some(c.into())) == 1;
    let cchk1 = |c: CardId, duel: &Duel| c.is_code(duel, ALPHA) && sg.is_exists(duel, cchk2, 1, None);
    (
        aux::chkf_mmz(tp, 1)(duel, sg, sg, None).0 && sg.is_exists(duel, cchk1, 1, None),
        false,
    )
}

fn cfilter(code: u32) -> impl Fn(CardId, &Duel) -> bool {
    move |c, duel| c.is_code(duel, code) && c.is_able_to_remove_as_cost(duel) && aux::sp_elim_filter(c, duel, true, true)
}

fn cost_group(duel: &Duel, tp: u8, c: CardId, code: u32) -> CardSet {
    duel.get_matching_group(
        cfilter(code),
        tp,
        LOCATION_MZONE | LOCATION_GRAVE | LOCATION_HAND,
        0,
        Some(c.into()),
    )
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    // The Lua shadows `c` with the handler, so the nil check never holds.
    let c = a.e.get_handler(duel);
    let tp = c.get_controler(duel);
    let (rg1, rg2, rg3) = (
        cost_group(duel, tp, c, ALPHA),
        cost_group(duel, tp, c, BETA),
        cost_group(duel, tp, c, GAMMA),
    );
    let rg = rg1.union(rg2).union(rg3);
    duel.get_location_count(tp, LOCATION_MZONE) > -3
        && !rg1.is_empty()
        && !rg2.is_empty()
        && !rg3.is_empty()
        && aux::select_unselect_group_check(duel, rg, 3, 3, &|duel, sg, _, _| rescon(duel, sg, tp))
}

fn sptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    let rg = cost_group(duel, tp, c, ALPHA)
        .union(cost_group(duel, tp, c, BETA))
        .union(cost_group(duel, tp, c, GAMMA));
    let g = aux::select_unselect_group_full(
        duel,
        rg,
        3,
        3,
        &|duel, sg, _, _| rescon(duel, sg, tp),
        tp,
        HINTMSG_REMOVE,
        None,
        true,
    )?;
    if !g.is_empty() {
        a.e.set_label_object_group(duel, Some(g));
        return Ok(true);
    }
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(g) = a.e.get_label_object_group(duel) {
        duel.remove(g, POS_FACEUP, REASON_COST)?;
    }
    Ok(())
}

fn costfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel)
        && (c.is_set_card(duel, &[SET_MAGNET_WARRIOR]) || [99785935, 39256679, 11549357].into_iter().any(|code| c.is_code(duel, code)))
        && c.is_level_below(duel, 4)
        && c.is_able_to_remove_as_cost(duel)
        && aux::sp_elim_filter(c, duel, true, false)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::banish_cost(duel, a, chk, costfilter, LOCATION_MZONE | LOCATION_GRAVE)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, 1 - tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

fn spcon2(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_reason(duel, REASON_BATTLE) || (a.rp != a.tp && c.is_reason(duel, REASON_EFFECT) && c.is_previous_controler(duel, a.tp))
}

fn spfilter(e: EffectId, tp: u8, code: u32) -> impl Fn(CardId, &Duel) -> bool {
    move |c, duel| c.is_faceup(duel) && c.is_code(duel, code) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg2(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 2
            && [ALPHA, BETA, GAMMA]
                .into_iter()
                .all(|code| duel.is_existing_target(spfilter(e, tp, code), tp, LOCATION_REMOVED, 0, 1, None))
            && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none());
    }
    let mut g1 = CardSet::default();
    for code in [ALPHA, BETA, GAMMA] {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        g1.merge(duel.select_matching_card(tp, spfilter(e, tp, code), tp, LOCATION_REMOVED, 0, 1, 1, None)?);
    }
    duel.set_target_card(g1);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(g1), 3, 0, 0);
    Ok(false)
}

fn spop2(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    let mut g = duel.get_target_cards(a.e).unwrap_or_default();
    if ft <= 0 || g.is_empty() || g.len() > 1 && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        return Ok(());
    }
    if g.len() as i32 <= ft {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    } else {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = g.select(duel, tp, ft as u8, ft as u8, None)?;
        duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
        g = g.difference(sg);
        duel.send_to_grave(g, REASON_RULE)?;
    }
    Ok(())
}
