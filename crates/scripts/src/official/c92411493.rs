// The Weather Painter Rain
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

use super::c54178659::{spcon, spreg, sptg};

const ID: u32 = 92411493;

pub const LISTED_SERIES: &[u16] = &[SET_THE_WEATHER];
// Banished as the cost of the negation The Weather Painter Rainbow grants to copies on the field, with Pot of Greed and Elfnote Lucina on
// the opponent's side to chain. The placing effect needs a "The Weather" Spell/Trap in the hand, and none is ported.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (54178659, MZONE),
    (54178659, MZONE),
    (13597785, MZONE | OPPONENT),
    (55144522, HAND | OPPONENT),
    (55144522, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0))),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, tftg);
    e1.set_operation(duel, tfop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e2.set_code(duel, EVENT_REMOVE);
    e2.set_operation(duel, spreg::<ID>);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_range(duel, LOCATION_REMOVED);
    e3.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e3.set_condition(duel, spcon::<ID>);
    e3.set_target(duel, sptg::<ID>);
    e3.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    e3.set_label_object(duel, Some(e2));
    c.register_effect(duel, e3);
}

fn tffilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_spell_trap(duel)
        && c.is_set_card(duel, &[SET_THE_WEATHER])
        && !c.is_forbidden(duel)
        && c.check_unique_on_field(duel, tp)
        && !c.is_type(duel, TYPE_FIELD)
}

fn tftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_matching_card(|c, duel| tffilter(c, duel, tp), tp, LOCATION_HAND, 0, 1, None));
    }
    Ok(false)
}

fn tfop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.get_location_count(tp, LOCATION_SZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    let g = duel.select_matching_card(tp, |c, duel| tffilter(c, duel, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        duel.move_to_field(tc, tp, tp, LOCATION_SZONE, POS_FACEUP, true)?;
    }
    Ok(())
}
