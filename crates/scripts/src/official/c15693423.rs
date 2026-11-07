// Evenly Matched
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 15693423;

// Two monsters of the opponent's, for the activation in the Battle Phase against an empty field.
pub const SUPPORT: &[(u32, u16)] = &[(15025844, MZONE | OPPONENT), (79629370, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, TIMING_BATTLE_END, TIMING_BATTLE_END);
    e1.set_condition(duel, |duel, _| duel.get_current_phase() == PHASE_BATTLE);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_TRAP_ACT_IN_HAND);
    e2.set_condition(duel, handcon);
    c.register_effect(duel, e2);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_field_group(tp, 0, LOCATION_ONFIELD);
    let mut ct = g.len() as i32 - duel.get_field_group_count(tp, LOCATION_ONFIELD, 0) as i32;
    if a.e.get_handler(duel).is_location(duel, LOCATION_HAND) {
        ct -= 1;
    }
    Ok(chk == 0
        && duel.is_player_affected_by_effect(1 - tp, 30459350).is_none()
        && ct > 0
        && g.is_exists(
            duel,
            |c, duel| c.is_able_to_remove_full(duel, Some(1 - tp), POS_FACEDOWN, REASON_RULE),
            1,
            None,
        ))
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.is_player_affected_by_effect(1 - tp, 30459350).is_some() {
        return Ok(());
    }
    let g = duel.get_field_group(tp, 0, LOCATION_ONFIELD);
    let ct = g.len() as i32 - duel.get_field_group_count(tp, LOCATION_ONFIELD, 0) as i32;
    if ct > 0 {
        duel.hint(HINT_SELECTMSG, 1 - tp, HINTMSG_REMOVE);
        let filter = |c: CardId, duel: &Duel| c.is_able_to_remove_full(duel, Some(1 - tp), POS_FACEDOWN, REASON_RULE);
        let sg = g.filter_select(duel, 1 - tp, filter, ct as u8, ct as u8, None)?;
        duel.remove_full(sg, POS_FACEDOWN, REASON_RULE, PLAYER_NONE, Some(1 - tp))?;
    }
    Ok(())
}

fn handcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.e.get_handler_player(duel);
    duel.get_field_group_count(tp, LOCATION_ONFIELD, 0) == 0 && duel.get_field_group_count(1 - tp, LOCATION_ONFIELD, 0) > 1
}
