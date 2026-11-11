// Prufinesse, the Tactical Trapper
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT, REMOVED};

const ID: u32 = 9798352;

// Two in play and in hand, Waboku in the Deck and Graveyard to banish, and banished Normal Traps (Trap Hole) to Set after
// the opponent's Raigeki destroys it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (12607053, DECK),
    (12607053, DECK),
    (12607053, DECK),
    (12607053, GRAVE),
    (4206964, REMOVED),
    (4206964, REMOVED),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_LVCHANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, bdcost);
    e1.set_operation(duel, bdop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_REMOVE + CATEGORY_LVCHANGE);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, bgtg);
    e3.set_operation(duel, bgop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_category(duel, CATEGORY_SET);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP);
    e4.set_code(duel, EVENT_DESTROYED);
    e4.set_count_limit(duel, 1, (ID, 2));
    e4.set_condition(duel, stcon);
    e4.set_target(duel, sttg);
    e4.set_operation(duel, stop);
    c.register_effect(duel, e4);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_trap(duel) && c.is_able_to_remove_as_cost(duel)
}

fn bdcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::banish_cost(duel, a, chk, cfilter, LOCATION_DECK)
}

fn bdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        c.update_level(duel, 1, None, None);
    }
    Ok(())
}

fn bgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_trap(duel) && c.is_able_to_remove(duel)
}

fn bgtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && bgfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(bgfilter, tp, LOCATION_GRAVE, LOCATION_GRAVE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, bgfilter, tp, LOCATION_GRAVE, LOCATION_GRAVE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, 0, 0);
    Ok(false)
}

fn bgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && duel.remove(tc, POS_FACEUP, REASON_EFFECT)? > 0
    {
        c.update_level(duel, 1, None, None);
    }
    Ok(())
}

fn stcon(duel: &Duel, a: &Args) -> bool {
    a.rp == 1 - a.tp && a.e.get_handler(duel).is_previous_controler(duel, a.tp)
}

fn stfilter(c: CardId, duel: &Duel) -> bool {
    c.is_normal_trap(duel) && c.is_faceup(duel) && c.is_ssetable(duel)
}

fn sttg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(
            duel.get_location_count(tp, LOCATION_SZONE) > 0 && duel.is_existing_matching_card(stfilter, tp, LOCATION_REMOVED, 0, 1, None)
        );
    }
    Ok(false)
}

fn stop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.get_location_count(tp, LOCATION_SZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let g = duel.select_matching_card(tp, stfilter, tp, LOCATION_REMOVED, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.sset(tp, g)?;
    }
    Ok(())
}
