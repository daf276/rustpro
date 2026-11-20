// Sandwich Day
use crate::prelude::*;
use crate::testing::{GRAVE, HAND};

const ID: u32 = 83838727;

// Copies in the hand, with a monster in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (26202165, GRAVE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DRAW + CATEGORY_HANDES);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(CardId::is_monster, a.tp, LOCATION_GRAVE, 0, 1, None)
    });
    e1.set_cost(duel, Cost::pay_lp::<200>);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 1));
    }
    duel.set_target_player(tp);
    duel.set_target_param(1);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    duel.set_possible_operation_info(0, CATEGORY_HANDES, None, 0, tp, 1);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    if duel.draw(p, d, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let Some(dc) = duel.get_operated_group().get_first() else {
        return Ok(());
    };
    duel.confirm_cards(1 - tp, dc)?;
    duel.shuffle_hand(tp)?;
    if !dc.is_monster(duel) {
        return Ok(());
    }
    duel.break_effect()?;
    let attribute = dc.get_attribute(duel);
    if !duel.is_existing_matching_card(|c, duel| c.is_attribute(duel, attribute), tp, LOCATION_GRAVE, 0, 1, None) {
        duel.draw(tp, 1, REASON_EFFECT)?;
    } else {
        duel.discard_hand(tp, aux::TRUE, 1, 1, REASON_EFFECT | REASON_DISCARD)?;
    }
    Ok(())
}
