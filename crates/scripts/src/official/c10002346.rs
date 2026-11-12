// Gachi Gachi Gantetsu
use super::c71594310::Procedure;
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE, OPPONENT};
use engine::constants::MSG_SELECT_EFFECTYN;

const ID: u32 = 10002346;

// In the Extra Deck, Xyz Summoned with Level 2 Normal Monsters (Doron, Flame Viper, Lucky Trinket), with Raigeki and Dark Hole
// on the opponent's side to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (756652, MZONE),
    (2830619, MZONE),
    (3985011, MZONE),
    (3985011, MZONE),
    (12580477, DECK | OPPONENT),
    (12580477, DECK | OPPONENT),
    (53129443, DECK | OPPONENT),
];
// The continuous effects resolve no chain link: the replacement prompt is what the card does.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::Message(MSG_SELECT_EFFECTYN)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure<2, 2>>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_DESTROY_REPLACE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target(duel, reptg);
    e1.set_operation(duel, repop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_value(
        duel,
        Value::Card(|duel, e, _| e.get_handler(duel).get_overlay_count(duel) as i32 * 200),
    );
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e3);
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(!c.is_reason(duel, REASON_REPLACE) && c.check_remove_overlay_card(duel, tp, 1, REASON_EFFECT));
    }
    duel.select_effect_yes_no_full(tp, c, 96)
}

fn repop(duel: &mut Duel, a: &Args) -> Script<()> {
    a.e.get_handler(duel).remove_overlay_card(duel, a.tp, 1, 1, REASON_EFFECT)?;
    Ok(())
}
