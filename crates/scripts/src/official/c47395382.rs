// Fabled Ragin
use super::c54048462::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 47395382;

// In the Extra Deck, Synchro Summoned with Fabled Raven (a Level 2 Tuner that discards the hand) and Swordsman of Landstar (Level 3).
pub const SUPPORT: &[(u32, u16)] = &[
    (47217354, MZONE),
    (47217354, MZONE),
    (47217354, MZONE),
    (3573512, MZONE),
    (3573512, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DRAW);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, con);
    e1.set_target(duel, tg);
    e1.set_operation(duel, op);
    c.register_effect(duel, e1);
}

fn con(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_synchro_summoned(duel)
}

fn tg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let h = duel.get_field_group_count(tp, LOCATION_HAND, 0) as i32;
    if chk == 0 {
        return Ok(h < 2 && duel.is_player_can_draw(tp, (2 - h) as usize));
    }
    duel.set_target_player(tp);
    duel.set_target_param((2 - h) as u32);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 2 - h);
    Ok(false)
}

fn op(duel: &mut Duel, _: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let h = duel.get_field_group_count(p, LOCATION_HAND, 0) as i32;
    if h >= 2 {
        return Ok(());
    }
    duel.draw(p, (2 - h) as u32, REASON_EFFECT)?;
    Ok(())
}
