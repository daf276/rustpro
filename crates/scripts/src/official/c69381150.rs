// A.I.Q
use engine::constants::MSG_SELECT_EFFECTYN;

use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 69381150;

pub const LISTED_SERIES: &[u16] = &[SET_IGNISTER];

// In the hand with Wizard @Ignister on the field, and Link Monsters on the field to Tribute and in the Extra Deck to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (3723262, MZONE),
    (3723262, MZONE),
    (74567889, MZONE),
    (74567889, EXTRA),
    (74567889, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::Message(MSG_SELECT_EFFECTYN)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER);
    e1.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(cfilter, a.tp, LOCATION_MZONE, 0, 1, None)
    });
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_condition(duel, |duel, a| a.eg.unwrap_or_default().is_exists(duel, is_link_summoned, 1, None));
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_property(duel, EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_CANNOT_DISABLE);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e3.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_IGNISTER])
}

fn is_link_summoned(c: CardId, duel: &Duel) -> bool {
    c.is_summon_type(duel, &[SUMMON_TYPE_LINK])
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let players =
        a.eg.unwrap_or_default()
            .filter(duel, is_link_summoned, None)
            .get_class(duel, |c, duel| i64::from(c.get_summon_player(duel)));
    let c = a.e.get_handler(duel);
    for p in players {
        let p = p as u8;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
        e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
        e1.set_absolute_range(duel, 0, u32::from(1 - p), u32::from(p));
        e1.set_range(duel, LOCATION_SZONE);
        e1.set_target_spsummon(duel, |_, _, _, _, sumtype, _, _, _| sumtype & SUMMON_TYPE_LINK == SUMMON_TYPE_LINK);
        e1.set_reset(duel, RESET_PHASE | PHASE_END | RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn desfilter(c: CardId, duel: &Duel) -> bool {
    c.is_releasable_by_effect(duel) && c.is_type(duel, TYPE_LINK)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    duel.hint_selection(c);
    if duel.check_release_group_full(tp, desfilter, 1, ReleaseOptions::new(tp)) && duel.select_effect_yes_no(tp, c)? {
        let g = duel.select_release_group_full(tp, desfilter, (1, 1), false, ReleaseOptions::new(tp))?;
        duel.release(g, REASON_COST)?;
    } else {
        duel.destroy(c, REASON_COST)?;
    }
    Ok(())
}
