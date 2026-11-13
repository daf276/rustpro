// Signal Warrior
use super::c44035031::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 9634146;

pub const COUNTER_PLACE_LIST: &[u16] = &[COUNTER_SIGNAL];

// Three copies on the field and one in the Extra Deck, so that Signal Counters pile up over a few Standby Phases.
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (ID, MZONE), (ID, MZONE), (ID, MZONE)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_COUNTER);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_operation(duel, ctop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).get_counter(duel, COUNTER_SIGNAL) > 0);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e3.set_value(duel, Value::EffectPlayer(aux::indoval));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, 0);
    e4.set_cost(duel, effcost);
    e4.set_operation(duel, effop);
    c.register_effect(duel, e4);
}

fn ctop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let mut g = duel.get_matching_group(CardId::is_faceup, 0, LOCATION_FZONE, LOCATION_FZONE, None);
    if c.is_relate_to_effect(duel, a.e) {
        g.insert(c);
    }
    for tc in g.iter() {
        if tc.is_can_add_counter(duel, COUNTER_SIGNAL, 1) {
            tc.add_counter(duel, COUNTER_SIGNAL, 1);
        }
    }
    Ok(())
}

fn effcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let b1 = duel.is_can_remove_counter(tp, 1, 1, COUNTER_SIGNAL, 4, REASON_COST);
    let b2 = duel.is_can_remove_counter(tp, 1, 1, COUNTER_SIGNAL, 7, REASON_COST) && duel.is_player_can_draw(tp, 1);
    let b3 = duel.is_can_remove_counter(tp, 1, 1, COUNTER_SIGNAL, 10, REASON_COST)
        && duel.is_existing_matching_card(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None);
    if chk == 0 {
        return Ok(b1 || b2 || b3);
    }
    let effects = [(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2)), (b3, aux::stringid(ID, 3))];
    let op = duel.select_effect(tp, &effects)?.unwrap_or(0);
    e.set_label(duel, op as i64);
    duel.remove_counter(tp, 1, 1, COUNTER_SIGNAL, 4 + (op as u16 - 1) * 3, REASON_COST)?;
    match op {
        1 => {
            e.set_category(duel, CATEGORY_DAMAGE);
            e.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
            duel.set_target_player(1 - tp);
            duel.set_target_param(800);
            duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 800);
        }
        2 => {
            e.set_category(duel, CATEGORY_DRAW);
            e.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
            duel.set_target_player(tp);
            duel.set_target_param(1);
            duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
        }
        3 => {
            e.set_category(duel, CATEGORY_DESTROY);
            e.set_property(duel, 0);
            let g = duel.get_matching_group(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
            duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, tp, 0);
        }
        _ => {}
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    match a.e.get_label(duel) {
        1 => stock::damage(duel, a)?,
        2 => stock::draw(duel, a)?,
        3 => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
            let g = duel.select_matching_card(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
            if !g.is_empty() {
                duel.hint_selection(g);
                duel.destroy(g, REASON_EFFECT)?;
            }
        }
        _ => {}
    }
    Ok(())
}
