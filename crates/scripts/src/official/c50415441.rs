// Mimighoul Archfiend
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 50415441;

pub const LISTED_SERIES: &[u16] = &[SET_MIMIGHOUL];
// Copies face-down on the field to Flip with the opponent's Elfnote Lucina to attack them, and in the hand to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, HAND),
    (ID, HAND),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DRAW + CATEGORY_TOGRAVE + CATEGORY_CONTROL);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_FLIP + EFFECT_TYPE_TRIGGER_F);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, _| duel.is_main_phase());
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, super::c81522098::selfsptg);
    e2.set_operation(duel, super::c81522098::selfspop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_POSITION);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_SUMMON_SUCCESS);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_target(duel, super::c55537983::postg);
    e3.set_operation(duel, super::c55537983::posop);
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e4);
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    if duel.is_player_can_draw(1 - tp, 1) {
        duel.set_operation_info(0, CATEGORY_DRAW, None, 0, 1 - tp, 1);
    }
    if duel.is_existing_matching_card(|c, duel| c.is_able_to_grave(duel), tp, LOCATION_HAND, 0, 1, None) {
        duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_HAND as i32);
    }
    let c = a.e.get_handler(duel);
    if c.is_able_to_change_controler(duel) {
        duel.set_operation_info(0, CATEGORY_CONTROL, Some(c.into()), 1, 0, 0);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.draw(1 - tp, 1, REASON_EFFECT)?;
    let mut break_chk = false;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, |c, duel| c.is_able_to_grave(duel), tp, LOCATION_HAND, 0, 1, 1, None)?;
    if !g.is_empty() {
        break_chk = true;
        duel.break_effect()?;
        duel.send_to_grave(g, REASON_EFFECT)?;
    }
    super::c81522098::give_control(duel, e, tp, break_chk)
}
