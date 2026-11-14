// Beware the White Forest
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 35778533;

pub const LISTED_SERIES: &[u16] = &[SET_WHITE_FOREST];
// Diabell (a Level 8 Illusion) on the field for the condition; copies in the hand for Astellar of the White Forest on the field
// to send as its cost.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (14307929, MZONE), (25592142, MZONE), (25592142, MZONE)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, descon);
    e1.set_target(duel, destg);
    e1.set_operation(duel, stock::destroy_first_target);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_LEAVE_GRAVE + CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_reason(duel, REASON_COST) && a.re.is_some_and(|re| re.is_activated(duel) && re.is_monster_effect(duel))
    });
    e2.set_target(duel, super::c99289828::settg);
    e2.set_operation(duel, super::c99289828::setop);
    c.register_effect(duel, e2);
}

fn desconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level_above(duel, 6) && c.is_race(duel, RACE_ILLUSION | RACE_SPELLCASTER) && c.is_faceup(duel)
}

fn descon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(desconfilter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, 1 - tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    Ok(false)
}
