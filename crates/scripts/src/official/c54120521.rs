// The Legend of Tickets
use crate::prelude::*;
use crate::testing::{GRAVE, HAND};

const ID: u32 = 54120521;

struct DeckTopOperation;

impl aux::DelayedOperation for DeckTopOperation {
    fn operate(duel: &mut Duel, rg: CardSet, _: &Args) -> Script<()> {
        duel.hint_selection_full(rg, true);
        duel.send_to_deck(rg, PLAYER_NONE, SEQ_DECKTOP, REASON_EFFECT)?;
        Ok(())
    }
}

// In the hand, with Normal Monsters (Kozaky) in the Graveyard to place on top of the Deck.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (99171160, GRAVE), (99171160, GRAVE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TODECK);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_possible_operation_info(0, CATEGORY_TODECK, g, 1, tp, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
    {
        aux::delayed_operation::<DeckTopOperation>(duel, tc.into(), PHASE_END, ID, e, tp);
    }
    let turn_ct = duel.get_turn_count();
    let ct = if duel.is_turn_player(tp) && duel.is_phase(PHASE_DRAW) {
        2
    } else {
        1
    };
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_DRAW_COUNT);
    e1.set_target_range(duel, 1, 0);
    e1.set_label(duel, i64::from(turn_ct));
    e1.set_condition(duel, |duel, a| i64::from(duel.get_turn_count()) != a.e.get_label(duel));
    e1.set_value(duel, 2);
    e1.set_reset_full(duel, RESET_PHASE | PHASE_DRAW | RESET_SELF_TURN, ct);
    duel.register_effect(e1, tp);
    Ok(())
}
