// Reckless Greed
use crate::prelude::*;
use crate::testing::SZONE;

const ID: u32 = 37576645;

// Set, to activate on a later turn.
pub const SUPPORT: &[(u32, u16)] = &[(ID, SZONE), (ID, SZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, stock::draw_target::<2>);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.draw(p, d, REASON_EFFECT)?;
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_SKIP_DP);
    e1.set_target_range(duel, 1, 0);
    e1.set_reset_full(duel, RESET_PHASE | PHASE_END, 5);
    duel.register_effect(e1, a.tp);
    Ok(())
}
