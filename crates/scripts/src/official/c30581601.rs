// Yummy★Snatchy
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 30581601;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 1;
    const MAX: Option<usize> = Some(1);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_level_below(duel, 4)
            && c.is_attribute_full(duel, ATTRIBUTE_LIGHT, Some(lc), sumtype, tp)
            && c.is_race_full(duel, RACE_BEAST, Some(lc), sumtype, tp)
    }
}

// Link Summoned from Marshmao (Level 1 LIGHT Beast), then Synchro Summoning Cooky★Yummy Way with Marshmao and itself (a Link-1
// monster, treated as a Level 1 Tuner). No Yummy Field Spell is ported, so the first effect is only reported.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (10966439, MZONE),
    (10966439, MZONE),
    (10966439, MZONE),
    (67098897, EXTRA),
    (67098897, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0))),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, pltg);
    e1.set_operation(duel, plop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_hint_timing(
        duel,
        0,
        TIMING_MAIN_END | TIMING_BATTLE_START | TIMING_BATTLE_END | TIMINGS_CHECK_MONSTER,
    );
    e2.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_CHAIN);
    e2.set_condition(duel, |duel, a| {
        duel.is_main_phase() || (duel.is_turn_player(1 - a.tp) && duel.is_battle_phase())
    });
    e2.set_cost(duel, Cost::pay_lp::<100>);
    e2.set_target(duel, synchtg);
    e2.set_operation(duel, synchop);
    c.register_effect(duel, e2);
}

fn plfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_YUMMY]) && c.is_field_spell(duel) && !c.is_forbidden(duel)
}

fn pltg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && duel.is_existing_matching_card(plfilter, a.tp, LOCATION_HAND | LOCATION_DECK, 0, 1, None))
}

fn plop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    let g = duel.select_matching_card(tp, plfilter, tp, LOCATION_HAND | LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first() {
        duel.place_field_spell(sc, tp)?;
    }
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    e1.set_target_spsummon(duel, |duel, _, c, _, sumtype, _, _, _| {
        c.is_link_above(duel, 3) && c.is_link_monster(duel) && sumtype & SUMMON_TYPE_LINK == SUMMON_TYPE_LINK
    });
    duel.register_effect(e1, tp);
    Ok(())
}

fn synchrocheck(duel: &Duel, _: u8, sg: CardSet, _: CardId) -> bool {
    sg.is_exists(duel, |c, duel| c.is_set_card(duel, &[SET_YUMMY]), 1, None)
}

fn synchtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        duel.set_synchro_check_additional(Some(synchrocheck));
        let res = duel.is_existing_matching_card(|c, duel| c.is_synchro_summonable(duel, None, None), tp, LOCATION_EXTRA, 0, 1, None);
        duel.set_synchro_check_additional(None);
        return Ok(res);
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn synchop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.set_synchro_check_additional(Some(synchrocheck));
    let g = duel.get_matching_group(|c, duel| c.is_synchro_summonable(duel, None, None), tp, LOCATION_EXTRA, 0, None);
    if g.is_empty() {
        duel.set_synchro_check_additional(None);
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    if let Some(sc) = g.select(duel, tp, 1, 1, None)?.get_first() {
        duel.synchro_summon(tp, sc, None, None)?;
    }
    Ok(())
}
