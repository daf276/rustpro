// Trap Trick
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 80101899;

pub const LISTED_NAMES: &[u32] = &[ID];

// Trap Hole, a Normal Trap, in the Deck: 4 copies, since one is banished and another Set.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (4206964, DECK), (4206964, DECK), (4206964, DECK), (4206964, DECK)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn rmfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_normal_trap(duel)
        && !c.is_code(duel, ID)
        && c.is_able_to_remove(duel)
        && duel.is_existing_matching_card(
            |sc, duel| setfilter(sc, duel, c.get_code(duel)),
            tp,
            LOCATION_DECK,
            0,
            1,
            Some(c.into()),
        )
}

fn setfilter(c: CardId, duel: &Duel, code: u32) -> bool {
    c.is_code(duel, code) && c.is_ssetable(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| rmfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, |c, duel| rmfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(rc) = g.get_first()
        && duel.remove(rc, POS_FACEUP, REASON_EFFECT)? > 0
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
        let code = rc.get_code(duel);
        let sg = duel.select_matching_card(tp, |c, duel| setfilter(c, duel, code), tp, LOCATION_DECK, 0, 1, 1, None)?;
        if let Some(sc) = sg.get_first()
            && duel.sset(tp, sc)? > 0
        {
            let e1 = duel.create_effect(c);
            e1.set_description(duel, aux::stringid(ID, 2));
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
            e1.set_code(duel, EFFECT_TRAP_ACT_IN_SET_TURN);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            sc.register_effect(duel, e1);
        }
    }
    if !e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        return Ok(());
    }
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_FIELD);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_operation(duel, aclimit1);
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e2, tp);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_FIELD);
    e3.set_code(duel, EVENT_CHAIN_NEGATED);
    e3.set_operation(duel, aclimit2);
    e3.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e3, tp);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_type(duel, EFFECT_TYPE_FIELD);
    e4.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e4.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e4.set_target_range(duel, 1, 0);
    e4.set_condition(duel, |duel, a| duel.get_flag_effect(a.e.get_handler_player(duel), ID) > 0);
    e4.set_value(
        duel,
        Value::EffectPlayer(|duel, _, te, _| te.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && te.is_trap_effect(duel)),
    );
    e4.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e4, tp);
    Ok(())
}

fn is_own_trap_activation(duel: &Duel, a: &Args) -> bool {
    a.ep == a.tp
        && a.re
            .is_some_and(|re| re.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && re.is_trap_effect(duel))
}

fn aclimit1(duel: &mut Duel, a: &Args) -> Script<()> {
    if is_own_trap_activation(duel, a) {
        duel.register_flag_effect(a.tp, ID, RESET_PHASE | PHASE_END, 0, 1);
    }
    Ok(())
}

fn aclimit2(duel: &mut Duel, a: &Args) -> Script<()> {
    if is_own_trap_activation(duel, a) {
        duel.reset_flag_effect(a.tp, ID);
    }
    Ok(())
}
