// Kunai with Chain
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT, SZONE};

const ID: u32 = 37390589;

// Set, to be activated on Mechanicalchaser, or against Elfnote Lucina's attack.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (7359741, MZONE),
    (7359741, MZONE),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, TIMING_DAMAGE_STEP, TIMING_DAMAGE_STEP);
    e1.set_condition(duel, aux::stat_change_damage_step_condition);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    a.e.set_label(duel, 1);
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(e.get_label(duel) == 2 && chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && chkc.is_faceup(duel));
    }
    let ac = duel.get_attacker();
    let b1 = duel.check_event(EVENT_ATTACK_ANNOUNCE)
        && ac.is_some_and(|ac| {
            ac.is_controler(duel, 1 - tp)
                && ac.is_location(duel, LOCATION_MZONE)
                && ac.is_attack_pos(duel)
                && ac.is_can_change_position(duel)
                && ac.is_can_be_effect_target(duel, e)
        });
    let b2 = e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && duel.is_existing_target(CardId::is_faceup, tp, LOCATION_MZONE, 0, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(
        tp,
        &[
            (b1, aux::stringid(ID, 1)),
            (b2, aux::stringid(ID, 2)),
            (b1 && b2, aux::stringid(ID, 3)),
        ],
    )?;
    duel.set_target_param(op.map_or(0, |op| op as u32));
    e.set_category(duel, 0);
    if let Some(ac) = ac
        && matches!(op, Some(1 | 3))
    {
        e.set_category(duel, CATEGORY_POSITION);
        duel.set_target_card(ac);
        duel.set_operation_info(0, CATEGORY_POSITION, Some(ac.into()), 1, tp, 0);
    }
    if matches!(op, Some(2 | 3)) {
        e.set_category(duel, e.get_category(duel) | CATEGORY_EQUIP);
        if e.get_label(duel) == 1 {
            aux::remain_field_cost(duel, a, 1)?;
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
        let g = duel.select_target(tp, CardId::is_faceup, tp, LOCATION_MZONE, 0, 1, 1, None)?;
        e.set_label_object_group(duel, g.and_then(|g| g.get_first()).map(CardSet::from));
        duel.set_operation_info(0, CATEGORY_EQUIP, Some(e.get_handler(duel).into()), 1, 0, 0);
    }
    e.set_label(duel, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let op: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    if op == 1 || op == 3 {
        // Lua errors on a nil attacker
        let tc = duel.get_attacker();
        unported!(duel, tc.is_none());
        if let Some(tc) = tc
            && tc.is_relate_to_effect(duel, e)
            && tc.is_faceup(duel)
            && tc.can_attack(duel)
            && !tc.is_status(duel, STATUS_ATTACK_CANCELED)
        {
            duel.change_position(tc, POS_FACEUP_DEFENSE)?;
        }
    }
    if op == 2 || op == 3 {
        let c = e.get_handler(duel);
        if !c.is_relate_to_effect(duel, e) || c.is_status(duel, STATUS_LEAVE_CONFIRMED) {
            return Ok(());
        }
        let tc = e.get_label_object_group(duel).and_then(|g| g.get_first());
        unported!(duel, tc.is_none());
        if let Some(tc) = tc
            && tc.is_relate_to_effect(duel, e)
            && tc.is_faceup(duel)
            && tc.is_controler(duel, tp)
            && duel.equip(tp, c, tc)?
        {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_EQUIP);
            e1.set_code(duel, EFFECT_UPDATE_ATTACK);
            e1.set_value(duel, 500);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            c.register_effect(duel, e1);
            let e2 = duel.create_effect(c);
            e2.set_type(duel, EFFECT_TYPE_SINGLE);
            e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
            e2.set_code(duel, EFFECT_EQUIP_LIMIT);
            e2.set_label(duel, tp.into());
            e2.set_value(
                duel,
                Value::Card(|duel, e, c| i32::from(c.is_controler(duel, e.get_label(duel) as u8))),
            );
            e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            c.register_effect(duel, e2);
        } else {
            c.cancel_to_grave_full(duel, false);
        }
    }
    Ok(())
}
