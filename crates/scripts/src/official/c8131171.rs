// Sinister Serpent
use crate::prelude::*;
use crate::testing::{GRAVE, MZONE};

const ID: u32 = 8131171;

pub const LISTED_NAMES: &[u32] = &[ID];

// Copies in the Graveyard and on the field to banish at the opponent's End Phase.
pub const SUPPORT: &[(u32, u16)] = &[(ID, GRAVE), (ID, GRAVE), (ID, MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_PHASE + PHASE_STANDBY);
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, a.tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_REMOVE, None, 1, a.tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) {
        duel.send_to_hand(c, PLAYER_NONE, REASON_EFFECT)?;
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_PHASE + PHASE_END);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, rmcon);
    e1.set_operation(duel, rmop);
    e1.set_reset(duel, RESET_PHASE | PHASE_END | RESET_OPPO_TURN);
    duel.register_effect(e1, tp);
    Ok(())
}

fn rmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, ID) && c.is_able_to_remove(duel) && aux::sp_elim_filter(c, duel, true, false)
}

fn rmcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    duel.is_turn_player(1 - tp) && duel.is_existing_matching_card(rmfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, None)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, rmfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}
