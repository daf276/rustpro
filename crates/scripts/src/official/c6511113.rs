// Traptrix Rafflesia
use engine::{Operation, Target};

use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 6511113;
const TRAP_HOLE: u32 = 4206964;

pub const LISTED_NAMES: &[u32] = &[ID];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

// On the field with Trap Hole in the Deck to copy and the opponent's Sangans to Normal Summon. No other "Traptrix" monster or
// "Hole" Normal Trap is ported.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (TRAP_HOLE, DECK),
    (TRAP_HOLE, DECK),
    (TRAP_HOLE, DECK),
    (26202165, HAND | OPPONENT),
    (26202165, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).get_overlay_count(duel) > 0);
    e1.set_value(duel, Value::Effect(|duel, _, te| te.is_trap_effect(duel)));
    c.register_effect(duel, e1);
    let e2a = duel.create_effect(c);
    e2a.set_type(duel, EFFECT_TYPE_FIELD);
    e2a.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e2a.set_range(duel, LOCATION_MZONE);
    e2a.set_target_range(duel, LOCATION_MZONE, 0);
    e2a.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_TRAPTRIX]) && !c.is_code(duel, ID));
    e2a.set_value(duel, 1);
    c.register_effect(duel, e2a);
    let e2b = e2a.clone_effect(duel);
    e2b.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    c.register_effect(duel, e2b);
    let e2c = e2a.clone_effect(duel);
    e2c.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e2c.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e2c.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e2c);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_count_limit(duel, 1, 0);
    e3.set_hint_timing(
        duel,
        0,
        TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMING_BATTLE_START | TIMING_MSET | TIMINGS_CHECK_MONSTER_E,
    );
    e3.set_cost(duel, cost);
    e3.set_target(duel, efftg);
    e3.set_operation(duel, effop);
    c.register_effect(duel, e3);
}

fn copyfilter(c: CardId, duel: &Duel) -> bool {
    c.is_normal_trap(duel)
        && c.is_set_card(duel, &[SET_HOLE, SET_TRAP_HOLE])
        && c.is_able_to_grave_as_cost(duel)
        && c.check_activate_effect(duel, false, true).is_some()
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    Cost::and(duel, a, chk, &[Cost::detach_from_self::<1, 1>, effcost])
}

fn effcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    e.set_label(duel, -100);
    if chk == 0 {
        let g = duel.get_matching_group(copyfilter, tp, LOCATION_DECK, 0, None);
        e.set_label_object_group(duel, Some(g));
        return Ok(!g.is_empty());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = e.get_label_object_group(duel).unwrap_or_default();
    let sc = g.select(duel, tp, 1, 1, None)?;
    e.set_label_object_group(duel, Some(sc));
    duel.send_to_grave(sc, REASON_COST)?;
    Ok(false)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    // The copied effect's target is called with the chkc, which a Trap without a target never receives.
    unported!(duel, chkc.is_some());
    if chk == 0 {
        let res = e.get_label(duel) == -100;
        e.set_label(duel, 0);
        return Ok(res);
    }
    let Some(sc) = e.get_label_object_group(duel).and_then(|g| g.get_first()) else {
        return Ok(false);
    };
    let Some((te, event)) = sc.check_activate_effect_event(duel, true, true) else {
        unported!(duel, true);
        return Ok(false);
    };
    if let Some(event) = event {
        duel.set_current_chain_event(event);
    }
    e.set_label(duel, te.get_label(duel));
    e.set_label_object(duel, te.get_label_object(duel));
    match te.get_target(duel) {
        Target::None => {}
        Target::Action(tg) => {
            let (property, property2) = te.get_property_full(duel);
            e.set_property_full(duel, property, property2);
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
            te.set_label(duel, e.get_label(duel));
            te.set_label_object(duel, e.get_label_object(duel));
            duel.clear_operation_info(0);
        }
        other => unported!(duel, !matches!(other, Target::None)),
    }
    e.set_label(duel, 0);
    e.set_label_object(duel, Some(te));
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let Some(te) = e.get_label_object(duel) else { return Ok(()) };
    let op = te.get_operation(duel);
    unported!(duel, !matches!(op, None | Some(Operation::Event(_) | Operation::Promptless(_))));
    if let Some(Operation::Event(op) | Operation::Promptless(op)) = op {
        e.set_label(duel, te.get_label(duel));
        e.set_label_object(duel, te.get_label_object(duel));
        op(duel, a)?;
    }
    e.set_label(duel, 0);
    e.set_label_object(duel, None);
    Ok(())
}
