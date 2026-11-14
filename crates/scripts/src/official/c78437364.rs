// Gouki The Great Ogre
use super::c22510667::Procedure;
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};
use engine::constants::MSG_SELECT_EFFECTYN;

const ID: u32 = 78437364;

pub const LISTED_SERIES: &[u16] = &[SET_GOUKI];
// The destruction replacement is no chain link and moves no card; its prompt is the MSG_SELECT_EFFECTYN.
// On the field with Gouki Guts and Gouki Machine Suprex, and Raigeki on the opponent's side to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (7540107, MZONE),
    (29099860, MZONE),
    (7540107, MZONE),
    (12580477, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Message(MSG_SELECT_EFFECTYN)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure<2, 0>>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1.set_value(duel, Value::Card(|duel, _, c| -c.get_base_defense(duel).max(0)));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_DESTROY_REPLACE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target(duel, desreptg);
    e2.set_operation(duel, desrepop);
    c.register_effect(duel, e2);
}

fn repfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_controler(duel, tp)
        && c.is_location(duel, LOCATION_MZONE)
        && c.is_destructable_full(duel, e)
        && !c.is_status(duel, STATUS_DESTROY_CONFIRMED)
}

fn desreptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        let g = c.get_linked_group(duel);
        return Ok(!c.is_reason(duel, REASON_REPLACE) && g.is_exists(duel, |c, duel| repfilter(c, duel, e, tp), 1, None));
    }
    if !duel.select_effect_yes_no_full(tp, c, 96)? {
        return Ok(false);
    }
    let g = c.get_linked_group(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESREPLACE);
    let sg = g.filter_select(duel, tp, |c, duel| repfilter(c, duel, e, tp), 1, 1, None)?;
    e.set_label_object_group(duel, Some(sg));
    if let Some(tc) = sg.get_first() {
        tc.set_status(duel, STATUS_DESTROY_CONFIRMED, true);
    }
    Ok(true)
}

fn desrepop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) {
        tc.set_status(duel, STATUS_DESTROY_CONFIRMED, false);
        duel.destroy(tc, REASON_EFFECT | REASON_REPLACE)?;
    }
    Ok(())
}
