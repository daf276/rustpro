// Gold Sarcophagus
use crate::prelude::*;
use crate::testing::{DECK, HAND};

const ID: u32 = 75500286;

// Raigeki in the Deck to banish.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (12580477, DECK), (12580477, DECK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_remove, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, CardId::is_able_to_remove, tp, LOCATION_DECK, 0, 1, 1, None)?;
    let Some(tg) = g.get_first() else { return Ok(()) };
    duel.remove(tg, POS_FACEUP, REASON_EFFECT)?;
    if !e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        return Ok(());
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_range(duel, LOCATION_REMOVED);
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
    tg.register_effect(duel, e1);
    Ok(())
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let ct = e.get_label(duel);
    let c = e.get_handler(duel);
    c.set_turn_counter(duel, (ct + 1) as u16);
    if ct == 1 {
        duel.send_to_hand(c, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, c)?;
    } else {
        e.set_label(duel, 1);
    }
    Ok(())
}
