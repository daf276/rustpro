// Lightforce Sword
use crate::prelude::*;
use crate::testing::{HAND, SZONE};

const ID: u32 = 49587034;
const PYRO_CLOCK: u32 = 1082946;

// In the hand and Set on the field.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, SZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_END_PHASE);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(
            |c, duel| c.is_able_to_remove_full(duel, Some(tp), POS_FACEDOWN, REASON_EFFECT),
            tp,
            0,
            LOCATION_HAND,
            1,
            None,
        ));
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, 1 - tp, LOCATION_HAND as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_field_group(1 - tp, LOCATION_HAND, 0);
    let rs = g.random_select(duel, 1 - tp, 1);
    let Some(card) = rs.get_first() else { return Ok(()) };
    if duel.remove(card, POS_FACEDOWN, REASON_EFFECT)? > 0 && e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        let c = e.get_handler(duel);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e1.set_range(duel, LOCATION_REMOVED);
        e1.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
        e1.set_count_limit(duel, 1, 0);
        e1.set_reset_full(
            duel,
            RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_STANDBY | RESET_SELF_TURN,
            5,
        );
        e1.set_condition(duel, thcon);
        e1.set_operation(duel, thop);
        e1.set_label(duel, 0);
        card.register_effect(duel, e1);
        card.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
        let descnum = if tp == c.get_owner(duel) { 0 } else { 1 };
        let e3 = duel.create_effect(c);
        e3.set_type(duel, EFFECT_TYPE_SINGLE);
        e3.set_description(duel, aux::stringid(ID, descnum));
        e3.set_property(
            duel,
            EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_IGNORE_IMMUNE + EFFECT_FLAG_SET_AVAILABLE,
        );
        e3.set_code(duel, PYRO_CLOCK);
        e3.set_label_object(duel, Some(e1));
        e3.set_owner_player(duel, tp);
        e3.set_operation(duel, reset_op);
        e3.set_reset_full(duel, RESET_PHASE | PHASE_STANDBY | RESET_OPPO_TURN, 4);
        c.register_effect(duel, e3);
    }
    Ok(())
}

fn reset_op(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(e1) = a.e.get_label_object(duel) else {
        a.e.reset(duel);
        return Ok(());
    };
    thop(
        duel,
        &Args {
            e: e1,
            re: Some(a.e),
            ..*a
        },
    )
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(a.tp) && a.e.get_handler(duel).get_flag_effect(duel, ID) == 0
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let ct = e.get_label(duel) + 1;
    e.get_owner(duel).set_turn_counter(duel, ct as u16);
    e.set_label(duel, ct);
    if ct == 4 {
        duel.send_to_hand(e.get_handler(duel), PLAYER_NONE, REASON_EFFECT)?;
        if let Some(re) = a.re {
            re.reset(duel);
        }
    }
    Ok(())
}
