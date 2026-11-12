// Dimensional Allotrope Varis
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 52254878;

// On the field with Elfnote Lucina on the opponent's side to attack it.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (13597785, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_value(duel, Value::Card(indval));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
}

fn indval(duel: &Duel, e: EffectId, c: CardId) -> i32 {
    let handler = e.get_handler(duel);
    i32::from(c.is_attribute(duel, handler.get_attribute(duel)) || c.is_race(duel, handler.get_race(duel)))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(true);
    }
    let c = e.get_handler(duel);
    let rc = duel.announce_race(tp, 1, RACE_ALL)?;
    let att = if c.is_race_except(duel, rc) {
        duel.announce_attribute(tp, 1, ATTRIBUTE_ALL)?
    } else {
        c.announce_another_attribute(duel, tp)?
    };
    e.set_labels(duel, &[rc as i64, i64::from(att)]);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if !c.is_faceup(duel) && c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let &[rc, att] = e.get_labels(duel) else { return Ok(()) };
    if c.is_race_except(duel, rc as u64) {
        change(duel, c, EFFECT_CHANGE_RACE, rc);
    }
    if c.is_attribute_except(duel, att as u32) {
        change(duel, c, EFFECT_CHANGE_ATTRIBUTE, att);
    }
    Ok(())
}

fn change(duel: &mut Duel, c: CardId, code: u32, value: i64) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, code);
    e1.set_property(duel, EFFECT_FLAG_COPY_INHERIT);
    e1.set_value(duel, value as i32);
    e1.set_reset_full(duel, RESETS_STANDARD_DISABLE_PHASE_END, 2);
    c.register_effect(duel, e1);
}
