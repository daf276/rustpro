// Transaction Rollback
use engine::{Operation, Target};

use crate::prelude::*;
use crate::testing::{Check, GRAVE, OPPONENT, SZONE};

const ID: u32 = 6351147;
const TYPHOON: u32 = 14883228;

pub const LISTED_NAMES: &[u32] = &[ID];

// Set on the field with Typhoon (Normal Trap) in each Graveyard to copy and Spells/Traps for it to target, and a copy in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (ID, GRAVE),
    (TYPHOON, GRAVE),
    (TYPHOON, GRAVE | OPPONENT),
    (TYPHOON, SZONE | OPPONENT),
    (ID, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    let timing = TIMING_DRAW_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E;
    e1.set_hint_timing(duel, timing, timing);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, cost);
    e1.set_target(duel, |duel, a, chk, chkc| target(duel, a, chk, chkc, false));
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_cost(duel, gycost);
    e2.set_target(duel, |duel, a, chk, chkc| target(duel, a, chk, chkc, true));
    c.register_effect(duel, e2);
}

fn paylp(duel: &mut Duel, tp: u8) -> Script<()> {
    let lp = duel.get_lp(tp) / 2;
    duel.pay_lp_cost(tp, lp as u32)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    paylp(duel, a.tp)?;
    Ok(false)
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_normal_trap(duel) && !c.is_code(duel, ID) && c.check_activate_effect(duel, false, true).is_some()
}

fn gycost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_remove_as_cost(duel));
    }
    paylp(duel, a.tp)?;
    duel.remove(c, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>, own: bool) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        let Some(te) = e.get_label_object(duel) else {
            // The Lua errors on the nil effect.
            unported!(duel, true);
            return Ok(false);
        };
        return match te.get_target(duel) {
            Target::None => Ok(false),
            Target::Action(tg) => tg(duel, a, 0, Some(chkc)),
            _ => {
                unported!(duel, true);
                Ok(false)
            }
        };
    }
    let (location1, location2) = if own { (LOCATION_GRAVE, 0) } else { (0, LOCATION_GRAVE) };
    let handler = own.then(|| a.e.get_handler(duel).into());
    if chk == 0 {
        return Ok(duel.is_existing_target(tgfilter, tp, location1, location2, 1, handler));
    }
    e.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let Some(sc) = duel
        .select_target(tp, tgfilter, tp, location1, location2, 1, 1, handler)?
        .and_then(|g| g.get_first())
    else {
        return Ok(false);
    };
    let Some((te, event)) = sc.check_activate_effect_event(duel, false, true) else {
        // The Lua errors on the nil effect.
        unported!(duel, true);
        return Ok(false);
    };
    if let Some(event) = event {
        duel.set_current_chain_event(event);
    }
    duel.clear_target_card();
    sc.create_effect_relation(duel, e);
    let tg = te.get_target(duel);
    let (property, property2) = te.get_property_full(duel);
    e.set_property_full(duel, property, property2);
    match tg {
        Target::None => {}
        Target::Action(tg) => {
            let args = Args {
                e,
                tp,
                eg: event.and_then(|event| event.event_cards),
                ep: event.map_or(0, |event| event.event_player),
                ev: event.map_or(0, |event| event.event_value),
                re: event.and_then(|event| event.reason_effect),
                r: event.map_or(0, |event| event.reason),
                rp: event.map_or(0, |event| event.reason_player),
                ..Args::default()
            };
            tg(duel, &args, 1, None)?;
        }
        other => unported!(duel, !matches!(other, Target::None)),
    }
    let label_object = e.get_label_object(duel);
    te.set_label_object(duel, label_object);
    e.set_label_object(duel, Some(te));
    duel.clear_operation_info(0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let Some(te) = e.get_label_object(duel) else { return Ok(()) };
    if !te.get_handler(duel).is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let label_object = te.get_label_object(duel);
    e.set_label_object(duel, label_object);
    let op = te.get_operation(duel);
    unported!(duel, !matches!(op, None | Some(Operation::Event(_) | Operation::Promptless(_))));
    if let Some(Operation::Event(op) | Operation::Promptless(op)) = op {
        op(duel, a)?;
    }
    Ok(())
}
