// Runick Golden Droplet
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};
use engine::constants::MSG_DRAW;

const ID: u32 = 20618850;

pub const LISTED_SERIES: &[u16] = &[SET_RUNICK];

struct Quickplay;

impl MysteruneQuickPlay for Quickplay {
    const ID: u32 = ID;
    const UNIQUECAT: u64 = CATEGORY_DRAW;
    const RMCOUNT: usize = 4;
    const UNIQUEPROP: Option<u32> = Some(EFFECT_FLAG_PLAYER_TARGET);

    fn uniquetg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
        drtg(duel, a, chk)
    }

    fn uniqueop(duel: &mut Duel, _: &Args) -> Script<bool> {
        drop(duel)
    }
}

// Hugin the Runick Wings in the Extra Deck to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (55990317, EXTRA)];
// The draw path shows only as the opponent's draw, a message every draw counts.
pub const CHECKS: &[Check] = &[Check::Message(MSG_DRAW), Check::SpecialSummonedCode(55990317)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Mysterune::create_qp_effect::<Quickplay>(duel, c);
    c.register_effect(duel, e1);
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_draw(1 - tp, 1) && duel.get_field_group_count(tp, 0, LOCATION_DECK) > 4);
    }
    duel.set_target_player(1 - tp);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, 1 - tp, 1);
    Ok(false)
}

fn drop(duel: &mut Duel) -> Script<bool> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    Ok(duel.draw(p, 1, REASON_EFFECT)? > 0)
}
