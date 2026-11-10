// DNA Surgery
use crate::prelude::*;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_CHANGE_RACE);
    e2.set_value(duel, Value::Card(|duel, e, _| e.get_label(duel) as i32));
    c.register_effect(duel, e2);
    e1.set_label_object(duel, Some(e2));
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(true);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RACE);
    let rc = duel.announce_race(tp, 1, RACE_ALL)?;
    if let Some(e2) = e.get_label_object(duel) {
        e2.set_label(duel, rc as i64);
    }
    e.get_handler(duel).set_hint(duel, CHINT_RACE, rc);
    Ok(false)
}
