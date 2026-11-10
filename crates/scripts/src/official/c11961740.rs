// Different Dimension Capsule
use crate::prelude::*;
use crate::testing::HAND;

const ID: u32 = 11961740;

pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_cost(duel, aux::remain_field_cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn removable(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_able_to_remove_full(duel, Some(tp), POS_FACEDOWN, REASON_EFFECT)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(a.e.is_has_type(duel, EFFECT_TYPE_ACTIVATE)
            && duel.is_existing_matching_card(|c, duel| removable(c, duel, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !(c.is_relate_to_effect(duel, e) && !c.is_status(duel, STATUS_LEAVE_CONFIRMED)) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, |c, duel| removable(c, duel, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first()
        && duel.remove(tc, POS_FACEDOWN, REASON_EFFECT)? != 0
        && e.is_has_type(duel, EFFECT_TYPE_ACTIVATE)
    {
        tc.register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD, 0, 1);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e1.set_range(duel, LOCATION_SZONE);
        e1.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
        e1.set_count_limit(duel, 1, 0);
        e1.set_reset_full(
            duel,
            RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_STANDBY | RESET_SELF_TURN,
            2,
        );
        e1.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
        e1.set_operation(duel, thop);
        e1.set_label(duel, 0);
        e1.set_label_object_group(duel, Some(tc.into()));
        c.register_effect(duel, e1);
    } else {
        c.cancel_to_grave_full(duel, false);
    }
    Ok(())
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    let ct = e.get_label(duel);
    c.set_turn_counter(duel, (ct + 1) as u16);
    if ct != 1 {
        e.set_label(duel, 1);
        return Ok(());
    }
    duel.destroy(c, REASON_RULE)?;
    if let Some(tc) = e.get_label_object_group(duel).and_then(|g| g.get_first())
        && tc.get_flag_effect(duel, ID) != 0
    {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
