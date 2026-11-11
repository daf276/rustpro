// Dragunity Javelin
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};
use engine::constants::MSG_EQUIP;

const ID: u32 = 80549379;

// Copies and Dragunity Tribus (Winged Beast) on the field, and Raigeki in the opponent's hand to destroy them.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (81962318, MZONE),
    (81962318, MZONE),
    (12580477, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Message(MSG_EQUIP)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_DESTROY_REPLACE);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_target(duel, reptg);
    e1.set_operation(duel, repop);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_DRAGUNITY]) && c.is_race(duel, RACE_WINGEDBEAST)
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0
            && !c.is_reason(duel, REASON_REPLACE)
            && c.is_location(duel, LOCATION_MZONE)
            && c.is_faceup(duel)
            && duel.is_existing_matching_card(filter, tp, LOCATION_MZONE, 0, 1, Some(c.into())));
    }
    duel.select_effect_yes_no_full(tp, c, 96)
}

fn repop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_MZONE, 0, 1, 1, Some(c.into()))?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    if duel.equip_full(tp, c, tc, false, false)? {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_EQUIP_LIMIT);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        e1.set_value(duel, Value::Card(super::c38423248::equip_limit));
        e1.set_label_object_group(duel, Some(tc.into()));
        c.register_effect(duel, e1);
    }
    Ok(())
}
