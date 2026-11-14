// Escalation of the Monarchs
use crate::prelude::*;
use crate::testing::{Check, HAND, SZONE};

const ID: u32 = 18235309;

// Copies on the field, with Monarchs and Mystical Sheep #2 in the hand to summon on the opponent's turn.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (4929256, HAND),
    (73125233, HAND),
    (83464209, HAND),
    (83464209, HAND),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_hint_timing(
        duel,
        0,
        TIMINGS_CHECK_MONSTER | TIMING_MAIN_END | TIMING_BATTLE_START | TIMING_BATTLE_END,
    );
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |duel, a| {
        duel.is_turn_player(1 - a.tp) && (duel.is_main_phase() || duel.is_battle_phase())
    });
    e3.set_target(duel, target2);
    e3.set_operation(duel, activate);
    c.register_effect(duel, e3);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_summonable_full(duel, true, None, 1, 0x1f) || c.is_msetable_full(duel, true, None, 1, 0x1f)
}

fn target2(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, a.tp, LOCATION_HAND, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SUMMON, None, 1, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !a.e.get_handler(duel).is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SUMMON);
    let Some(tc) = duel.select_matching_card(tp, filter, tp, LOCATION_HAND, 0, 1, 1, None)?.get_first() else {
        return Ok(());
    };
    let s1 = tc.is_summonable_full(duel, true, None, 1, 0x1f);
    let s2 = tc.is_msetable_full(duel, true, None, 1, 0x1f);
    if (s1 && s2 && duel.select_position(tp, tc, (POS_FACEUP_ATTACK + POS_FACEDOWN_DEFENSE) as u8)? == POS_FACEUP_ATTACK) || !s2 {
        duel.summon_full(tp, tc, true, None, 1, 0x1f)
    } else {
        duel.mset_full(tp, tc, true, None, 1, 0x1f)
    }
}
