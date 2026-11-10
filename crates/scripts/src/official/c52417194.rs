// Heavy Slump
use crate::prelude::*;
use crate::testing::{HAND, OPPONENT};

const ID: u32 = 52417194;

// Luster Dragons in the opponent's hand, for it to hold more than 7 cards.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (11091375, HAND | OPPONENT),
    (11091375, HAND | OPPONENT),
    (11091375, HAND | OPPONENT),
    (11091375, HAND | OPPONENT),
    (11091375, HAND | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TODECK + CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_hint_timing(duel, 0, TIMING_TOHAND);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    duel.get_field_group_count(a.tp, 0, LOCATION_HAND) > 7
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_player_can_draw(1 - a.tp, 0));
    }
    duel.set_target_player(1 - a.tp);
    Ok(false)
}

fn activate(duel: &mut Duel, _: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let g = duel.get_field_group(p, LOCATION_HAND, 0);
    duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    duel.shuffle_deck(p)?;
    duel.break_effect()?;
    duel.draw(p, 2, REASON_EFFECT)?;
    Ok(())
}
