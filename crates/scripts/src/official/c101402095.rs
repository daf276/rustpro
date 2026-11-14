// Angelechy Endgame Problem
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND};

const ID: u32 = 101402095;

pub const LISTED_SERIES: &[u16] = &[SET_ANGELECHY];
// In the hand with Angelechy Opposition and Angelechy Verdict to place Angelechy monsters from the Extra Deck in the Spell & Trap
// Zone.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (101402094, HAND),
    (101402094, HAND),
    (101402096, GRAVE),
    (101402096, GRAVE),
    (81797573, EXTRA),
    (55393975, EXTRA),
    (28904860, EXTRA),
    (42410161, EXTRA),
    (101402090, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EVENT_PHASE + PHASE_STANDBY);
    e1.set_range(duel, LOCATION_FZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp) && angelcountcondition::<1>(duel, a));
    e1.set_target(duel, stock::draw_target::<1>);
    e1.set_operation(duel, stock::draw);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EFFECT_OPPO_CHOOSES_SPSUMMON_ZONE);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_target_range(duel, 0, 1);
    e2.set_condition(duel, angelcountcondition::<2>);
    e2.set_oppo_chooses_zone_operation(duel, |_, _, _, _, r| r == LOCATION_REASON_SPSUMMON);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e3.set_code(duel, EFFECT_TRAP_ACT_IN_SET_TURN);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_target_range(duel, LOCATION_SZONE, 0);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, angelcountcondition::<3>);
    e3.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_ANGELECHY]) && c.is_trap(duel));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_category(duel, CATEGORY_REMOVE);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e4.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e4.set_range(duel, LOCATION_FZONE);
    e4.set_count_limit(duel, 1, 0);
    e4.set_condition(duel, |duel, a| a.rp == 1 - a.tp && angelcountcondition::<4>(duel, a));
    e4.set_target(duel, banishtg);
    e4.set_operation(duel, banishop);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 3));
    e5.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e5.set_type(duel, EFFECT_TYPE_IGNITION);
    e5.set_range(duel, LOCATION_FZONE);
    e5.set_count_limit(duel, 1, 0);
    e5.set_condition(duel, angelcountcondition::<5>);
    e5.set_target(duel, sptg);
    e5.set_operation(duel, spop);
    c.register_effect(duel, e5);
}

fn angelechyfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ANGELECHY]) && c.is_monster_card(duel) && c.is_faceup(duel)
}

fn angelcountcondition<const REQUIRED: usize>(duel: &Duel, a: &Args) -> bool {
    duel.get_matching_group_count(angelechyfilter, a.e.get_handler_player(duel), LOCATION_STZONE, 0, None) >= REQUIRED
}

fn banishtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let eg = a.eg.unwrap_or_default();
    let g = eg
        .filter(duel, CardId::is_able_to_remove, None)
        .filter(duel, |c, duel| c.is_summon_player(duel, 1 - tp), None);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    for sc in eg {
        sc.create_effect_relation(duel, e);
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(g), g.len() as u8, tp, 0);
    Ok(false)
}

fn banishop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g =
        a.eg.unwrap_or_default()
            .filter(duel, |c, duel| c.is_relate_to_effect(duel, e), None)
            .filter(duel, |c, duel| c.is_summon_player(duel, 1 - tp), None);
    if !g.is_empty() {
        duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_ANGELECHY])
        && duel.get_location_count_from_ex(tp, tp, None, c) > 0
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(
        duel,
        a,
        chk,
        |c, duel| spfilter(c, duel, a.e, a.tp),
        LOCATION_EXTRA,
        CATEGORY_SPECIAL_SUMMON,
    )
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
