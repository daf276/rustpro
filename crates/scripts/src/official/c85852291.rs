// Magical Mallet
use crate::prelude::*;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TODECK + CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 0)
            && duel.is_existing_matching_card(CardId::is_able_to_deck, tp, LOCATION_HAND, 0, 1, Some(e.get_handler(duel).into())));
    }
    duel.set_target_player(tp);
    duel.set_operation_info(0, CATEGORY_TODECK, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, _: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    duel.hint(HINT_SELECTMSG, p, HINTMSG_TODECK);
    let g = duel.select_matching_card(p, CardId::is_able_to_deck, p, LOCATION_HAND, 0, 1, 63, None)?;
    if g.is_empty() {
        return Ok(());
    }
    duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    duel.shuffle_deck(p)?;
    duel.break_effect()?;
    duel.draw(p, g.len() as u32, REASON_EFFECT)?;
    Ok(())
}
