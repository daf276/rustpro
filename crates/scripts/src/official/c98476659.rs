// Pot of Sloth
use engine::constants::MSG_SORT_CARD;

use crate::prelude::*;
use crate::testing::Check;

const ID: u32 = 98476659;

pub const LISTED_NAMES: &[u32] = &[ID];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::Message(MSG_SORT_CARD)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DRAW + CATEGORY_TODECK);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let draw_count = duel.get_field_group_count(tp, 0, LOCATION_ONFIELD);
    if chk == 0 {
        return Ok(draw_count > 0 && duel.is_player_can_draw(tp, draw_count));
    }
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, draw_count as i32);
    if draw_count >= 2 {
        duel.set_operation_info(0, CATEGORY_TODECK, None, (draw_count - 1) as u8, tp, LOCATION_HAND as i32);
    }
    duel.set_possible_operation_info(0, CATEGORY_TODECK, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let draw_count = duel.get_field_group_count(tp, 0, LOCATION_ONFIELD);
    if draw_count > 0 && duel.draw(tp, draw_count as u32, REASON_EFFECT)? > 0 && draw_count >= 2 {
        duel.shuffle_hand(tp)?;
        let return_count = (draw_count - 1) as u8;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
        let g = duel.select_matching_card(tp, CardId::is_able_to_deck, tp, LOCATION_HAND, 0, return_count, return_count, None)?;
        if !g.is_empty() {
            duel.break_effect()?;
            if duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKBOTTOM, REASON_EFFECT)? > 0 && return_count >= 2 {
                duel.sort_deckbottom(tp, tp, return_count.into())?;
            }
        }
    }
    if !e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        return Ok(());
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_target_range(duel, 1, 0);
    e1.set_value(duel, Value::EffectPlayer(|duel, _, re, _| re.get_handler(duel).is_code(duel, ID)));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}
