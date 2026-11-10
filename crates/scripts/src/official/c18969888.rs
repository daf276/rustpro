// Sangenpai Transcendent Dragion
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 18969888;

// In the Extra Deck, Synchro Summoned with Dragon Core Hexer (Level 8 Tuner) and Petit Dragon (Level 2).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (38955728, MZONE),
    (38955728, MZONE),
    (75356564, MZONE),
    (75356564, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_DRAGON, Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, |c, duel| c.is_race_full(duel, RACE_DRAGON, Some(sc), sumtype, tp))
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_POSITION);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e1.set_target(duel, postg);
    e1.set_operation(duel, posop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_MUST_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, 0, LOCATION_MZONE);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, 0, 1);
    e3.set_condition(duel, |duel, _| duel.is_battle_phase());
    e3.set_value(duel, 1);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_DESTROY);
    e4.set_type(duel, EFFECT_TYPE_QUICK_O);
    e4.set_code(duel, EVENT_FREE_CHAIN);
    e4.set_range(duel, LOCATION_GRAVE);
    e4.set_hint_timing(
        duel,
        0,
        TIMING_BATTLE_PHASE | TIMING_BATTLE_STEP_END | TIMING_BATTLE_END | TIMINGS_CHECK_MONSTER_E,
    );
    e4.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_DUEL);
    e4.set_condition(duel, |duel, _| duel.get_flag_effect(0, ID) + duel.get_flag_effect(1, ID) >= 3);
    e4.set_target(duel, sptg);
    e4.set_operation(duel, spop);
    c.register_effect(duel, e4);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_ATTACK_ANNOUNCE);
        ge1.set_operation(duel, |duel, a| {
            duel.register_flag_effect(a.ep, ID, RESET_PHASE | PHASE_END, 0, 1);
            Ok(())
        });
        duel.register_effect(ge1, 0);
    });
}

fn posfilter(c: CardId, duel: &Duel) -> bool {
    c.is_defense_pos(duel) && c.is_can_change_position(duel)
}

fn postg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(posfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    let g = duel.get_matching_group(posfilter, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    let count = g.len() as u8;
    duel.set_operation_info(0, CATEGORY_POSITION, Some(g), count, PLAYER_ALL, POS_FACEUP_ATTACK as i32);
    Ok(false)
}

fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(posfilter, a.tp, LOCATION_MZONE, LOCATION_MZONE, None);
    if !g.is_empty() {
        duel.change_position(g, POS_FACEUP_ATTACK)?;
    }
    Ok(())
}

pub fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, a.e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, None, 1, PLAYER_EITHER, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if c.is_relate_to_effect(duel, a.e) && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        let g = duel.get_field_group(0, LOCATION_ONFIELD, LOCATION_ONFIELD);
        if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
            let dg = g.select(duel, tp, 1, 1, None)?;
            duel.hint_selection_full(dg, true);
            duel.break_effect()?;
            duel.destroy(dg, REASON_EFFECT)?;
        }
    }
    Ok(())
}
