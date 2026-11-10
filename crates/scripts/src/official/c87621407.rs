// Dekoichi the Battlechanted Locomotive
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 87621407;
const BOKOICHI: u32 = 8715625;

pub const LISTED_NAMES: &[u32] = &[BOKOICHI];

// Face-down on the field to be flipped by an attack, with Bokoichi on the field to draw extra.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (BOKOICHI, MZONE), (26202165, MZONE | OPPONENT)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_FLIP);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_code(duel, BOKOICHI)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tp = a.tp;
    let ct = duel.get_matching_group_count(filter, tp, LOCATION_ONFIELD, 0, None);
    duel.set_target_player(tp);
    duel.set_target_param(ct as u32 + 1);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, ct as i32 + 1);
    Ok(false)
}

fn operation(duel: &mut Duel, _: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.draw(p, d, REASON_EFFECT)?;
    Ok(())
}
