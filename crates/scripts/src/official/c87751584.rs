// Gatling Dragon
use engine::constants::MSG_TOSS_COIN;

use crate::prelude::*;
use crate::testing::{Check, MZONE};

const ID: u32 = 87751584;

pub const TOSS_COIN: bool = true;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Code(81480460), FusionMaterial::Code(25551951)];
}

// A second copy on the field, so that the effect has monsters to destroy wherever this one starts.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE)];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::Message(MSG_TOSS_COIN)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_COIN);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(aux::TRUE, a.tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_COIN, None, 0, a.tp, 3);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(aux::TRUE, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    if g.is_empty() {
        return Ok(());
    }
    let results = duel.toss_coin(tp, 3)?;
    let ct = duel.count_heads(&results).min(g.len()) as u8;
    if ct == 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let dg = g.select(duel, tp, ct, ct, None)?;
    duel.hint_selection(dg);
    duel.destroy(dg, REASON_EFFECT)?;
    Ok(())
}
