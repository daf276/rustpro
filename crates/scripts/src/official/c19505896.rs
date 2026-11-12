// Nettles
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 19505896;

// Copies and Dark Plant (a Plant) on the field to be destroyed instead, with the opponent's Dark Hole and Raigeki.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (13193642, MZONE),
    (53129443, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Replaced];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_DESTROY_REPLACE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target(duel, desreptg);
    e1.set_operation(duel, desrepop);
    c.register_effect(duel, e1);
}

fn repfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_faceup(duel) && c.is_race(duel, RACE_PLANT) && c.is_destructable_full(duel, e) && !c.is_status(duel, STATUS_DESTROY_CONFIRMED)
}

fn desreptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(!c.is_reason(duel, REASON_REPLACE)
            && duel.is_existing_matching_card(|tc, duel| repfilter(tc, duel, e), tp, LOCATION_MZONE, 0, 1, Some(c.into())));
    }
    if !duel.select_effect_yes_no_full(tp, c, 96)? {
        return Ok(false);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESREPLACE);
    let g = duel.select_matching_card(tp, |tc, duel| repfilter(tc, duel, e), tp, LOCATION_MZONE, 0, 1, 1, Some(c.into()))?;
    e.set_label_object_group(duel, Some(g));
    if let Some(tc) = g.get_first() {
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
