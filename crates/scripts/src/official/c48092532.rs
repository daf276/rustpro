// D.D. Survivor
use crate::prelude::*;
use crate::testing::{HAND, MZONE};

const ID: u32 = 48092532;

// On the field to be banished face-up by Bystial Magnamhut in the hand.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (33854624, HAND), (33854624, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_REMOVE);
    e1.set_condition(duel, rmcon);
    e1.set_operation(duel, rmop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_PHASE + PHASE_END);
    e2.set_range(duel, LOCATION_REMOVED);
    e2.set_condition(duel, condition);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
}

fn rmcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_faceup(duel)
        && c.is_previous_position(duel, POS_FACEUP)
        && c.is_previous_location(duel, LOCATION_ONFIELD)
        && c.is_previous_controler(duel, a.tp)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    a.e.get_handler(duel)
        .register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
    Ok(())
}

fn condition(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).get_flag_effect(duel, ID) != 0
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.get_flag_effect(duel, ID + 1) == 0);
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    c.register_flag_effect(
        duel,
        ID + 1,
        RESET_EVENT
            | RESET_TURN_SET
            | RESET_TOGRAVE
            | RESET_TEMP_REMOVE
            | RESET_TOHAND
            | RESET_TODECK
            | RESET_OVERLAY
            | RESET_PHASE
            | PHASE_END,
        0,
        1,
    );
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    if c.is_relate_to_effect(duel, a.e) {
        if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
            duel.send_to_grave(c, REASON_EFFECT)?;
            return Ok(());
        }
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
