// Proxy Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};
use engine::constants::MSG_SELECT_EFFECTYN;

const ID: u32 = 22862454;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);
}

// With monsters on the field to be destroyed by the opponent's Hammer Shot or by Elfnote Lucina in battle and a linked one to destroy
// instead. The prompt is checked, as the monster it destroys is not the card under test.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (32295838, MZONE),
    (36211150, MZONE),
    (32295838, MZONE),
    (26412047, HAND | OPPONENT),
    (26412047, HAND | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Message(MSG_SELECT_EFFECTYN)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_DESTROY_REPLACE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target(duel, desreptg);
    e1.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(repfilter(c, duel, e.get_handler_player(duel)))),
    );
    e1.set_operation(duel, desrepop);
    c.register_effect(duel, e1);
}

fn repfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_controler(duel, tp)
        && c.is_location(duel, LOCATION_ONFIELD)
        && c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
        && !c.is_reason(duel, REASON_REPLACE)
}

fn desfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_controler(duel, tp)
        && c.is_monster(duel)
        && c.is_destructable_full(duel, e)
        && !c.is_status(duel, STATUS_DESTROY_CONFIRMED | STATUS_BATTLE_DESTROYED)
}

fn desreptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let g = c.get_linked_group(duel);
    if chk == 0 {
        return Ok(a.eg.unwrap_or_default().is_exists(duel, |c, duel| repfilter(c, duel, tp), 1, None)
            && g.is_exists(duel, |c, duel| desfilter(c, duel, e, tp), 1, None));
    }
    if !duel.select_effect_yes_no_full(tp, c, 96)? {
        return Ok(false);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESREPLACE);
    let sg = g.filter_select(duel, tp, |c, duel| desfilter(c, duel, e, tp), 1, 1, None)?;
    e.set_label_object_group(duel, Some(sg));
    duel.hint_selection(sg);
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
