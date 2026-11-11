// Morphtronic Accelerator
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 44424095;

// In the hand with Lantron to return and a monster of the opponent's to destroy.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (76865611, HAND), (26202165, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TODECK + CATEGORY_DESTROY + CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MORPHTRONIC]) && c.is_monster(duel) && c.is_able_to_deck(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let handler = e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc != handler);
    }
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_HAND, 0, 1, None)
            && duel.is_existing_target(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(handler.into()))
            && duel.is_player_can_draw(tp, 1));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let dg = duel.select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, Some(handler.into()))?;
    duel.set_operation_info(0, CATEGORY_DESTROY, dg, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_TODECK, None, 1, tp, LOCATION_HAND as i32);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND, 0, 1, 1, None)?;
    if g.is_empty() {
        return Ok(());
    }
    duel.confirm_cards(1 - tp, g)?;
    duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKTOP, REASON_EFFECT)?;
    duel.shuffle_deck(tp)?;
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.break_effect()?;
        if duel.destroy(tc, REASON_EFFECT)? == 0 {
            return Ok(());
        }
        duel.draw(tp, 1, REASON_EFFECT)?;
    }
    Ok(())
}
