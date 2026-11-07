// Solemn Accusation
use crate::prelude::*;
use crate::testing::{Check, HAND, OPPONENT, SZONE};

const ID: u32 = 78114463;

// Set, to be activated on Raigeki, which the opponent holds.
pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE), (12580477, HAND | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_description(duel, aux::stringid(ID, 0));
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e0.set_code(duel, EFFECT_TRAP_ACT_IN_SET_TURN);
    e0.set_value(duel, Value::Activation(|duel, e, _| e.set_label(duel, 1)));
    e0.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(
            face_down_trap,
            a.e.get_handler_player(duel),
            LOCATION_SZONE,
            0,
            1,
            Some(a.e.get_handler(duel).into()),
        )
    });
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_condition(duel, |duel, a| {
        a.re.is_some_and(|re| re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)) && duel.is_chain_negatable(a.ev as u8)
    });
    e1.set_cost(duel, |duel, a, chk| Cost::and(duel, a, chk, &[revealcost, paylpcost]));
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    e1.set_label_object(duel, Some(e0));
    c.register_effect(duel, e1);
}

fn face_down_trap(c: CardId, duel: &Duel) -> bool {
    c.is_facedown(duel) && c.is_trap(duel)
}

fn revealcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let Some(label_obj) = a.e.get_label_object(duel) else {
        return Ok(chk == 0);
    };
    if chk == 0 {
        label_obj.set_label(duel, 0);
        return Ok(true);
    }
    if label_obj.get_label(duel) > 0 {
        label_obj.set_label(duel, 0);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
        let g = duel.select_matching_card(tp, face_down_trap, tp, LOCATION_SZONE, 0, 1, 1, Some(a.e.get_handler(duel).into()))?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(false)
}

fn banish_activation_chk(duel: &Duel, a: &Args) -> bool {
    let Some(re) = duel.get_chain_info::<Option<EffectId>>(0, CHAININFO_TRIGGERING_EFFECT) else {
        return false;
    };
    let rc = re.get_handler(duel);
    rc.is_able_to_remove_full(duel, Some(a.tp), POS_FACEUP, REASON_EFFECT)
        || (!rc.is_relate_to_effect(duel, re) && duel.is_player_can_remove(a.tp, None))
}

fn paylpcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let choices: [CostChoice; 2] = [
        (Cost::pay_lp::<1500>, aux::stringid(ID, 2), |_, _| true),
        (Cost::pay_lp::<3000>, aux::stringid(ID, 3), banish_activation_chk),
    ];
    Cost::choice(duel, a, chk, &choices)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let (e, tp, eg) = (a.e, a.tp, a.eg);
    let Some(re) = a.re else { return Ok(false) };
    let rc = re.get_handler(duel);
    let relation = rc.is_relate_to_effect(duel, re);
    duel.set_operation_info(0, CATEGORY_NEGATE, eg, 1, tp, 0);
    match Cost::get_choice(duel, e) {
        Some(1) => {
            e.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
            if rc.is_destructable(duel) && relation {
                duel.set_operation_info(0, CATEGORY_DESTROY, eg, 1, tp, 0);
            }
        }
        Some(2) => {
            e.set_category(duel, CATEGORY_NEGATE + CATEGORY_REMOVE);
            if relation {
                duel.set_operation_info(0, CATEGORY_REMOVE, Some(rc.into()), 1, tp, 0);
            } else {
                duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, tp, rc.get_previous_location(duel) as i32);
            }
        }
        _ => {}
    }
    Ok(false)
}

fn banfilter(c: CardId, duel: &Duel, code: u32, opp: u8) -> bool {
    c.is_original_code_rule(duel, code) && c.is_able_to_remove_full(duel, Some(opp), POS_FACEUP, REASON_EFFECT)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    let rc = re.get_handler(duel);
    let code = rc.get_original_code_rule(duel);
    match Cost::get_choice(duel, a.e) {
        Some(1) => destroy_and_lock(duel, a, re, code),
        Some(2) => banish_all_copies(duel, a, re, code),
        _ => Ok(()),
    }
}

fn destroy_and_lock(duel: &mut Duel, a: &Args, re: EffectId, code: u32) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let rc = re.get_handler(duel);
    if !(duel.negate_activation(a.ev as u8)
        && rc.is_relate_to_effect(duel, re)
        && duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)? > 0)
    {
        return Ok(());
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 4));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_target_range(duel, 1, 1);
    e1.set_label(duel, i64::from(code));
    e1.set_value(
        duel,
        Value::EffectPlayer(|duel, e, re, _| re.get_handler(duel).is_original_code_rule(duel, e.get_label(duel) as u32)),
    );
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}

fn banish_all_copies(duel: &mut Duel, a: &Args, re: EffectId, code: u32) -> Script<()> {
    let tp = a.tp;
    let rc = re.get_handler(duel);
    if !(duel.negate_activation(a.ev as u8)
        && rc.is_relate_to_effect(duel, re)
        && duel.remove(a.eg.unwrap_or_default(), POS_FACEUP, REASON_EFFECT)? > 0)
    {
        return Ok(());
    }
    let opp = 1 - tp;
    let g = duel.get_matching_group(|c, duel| banfilter(c, duel, code, opp), tp, 0, LOCATION_HAND | LOCATION_DECK, None);
    if !g.is_empty() {
        duel.break_effect()?;
        duel.remove_full(g, POS_FACEUP, REASON_EFFECT, PLAYER_NONE, Some(opp))?;
    }
    Ok(())
}
