// Pot of Duality
use crate::prelude::*;

const ID: u32 = 98645731;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_activity_count(tp, ACTIVITY_SPSUMMON) == 0);
    }
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let g = duel.get_decktop_group(tp, 3);
        return Ok(g.len() == 3 && g.is_exists(duel, CardId::is_able_to_hand, 1, None));
    }
    duel.set_target_player(tp);
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, _: &Args) -> Script<()> {
    let affected_player: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let decktop3 = duel.get_decktop_group(affected_player, 3);
    if decktop3.is_empty() {
        return Ok(());
    }
    duel.confirm_decktop(affected_player, 3)?;
    let can_add = decktop3.is_exists(duel, CardId::is_able_to_hand, 1, None);
    duel.hint(
        HINT_SELECTMSG,
        affected_player,
        if can_add { HINTMSG_ATOHAND } else { HINTMSG_TOGRAVE },
    );
    let filter = |c: CardId, duel: &Duel| !can_add || c.is_able_to_hand(duel);
    let Some(sc) = decktop3.filter_select(duel, affected_player, filter, 1, 1, None)?.get_first() else {
        return Ok(());
    };
    if sc.is_able_to_hand(duel) {
        duel.send_to_hand(sc, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - affected_player, sc)?;
        duel.shuffle_hand(affected_player)?;
    } else {
        duel.send_to_grave(sc, REASON_RULE)?;
    }
    duel.break_effect()?;
    duel.shuffle_deck(affected_player)?;
    Ok(())
}
