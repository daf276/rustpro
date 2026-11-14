// Mimighoul Slime
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 80551022;

pub const LISTED_SERIES: &[u16] = &[SET_MIMIGHOUL];
// Copies face-down on the field to Flip with the opponent's Elfnote Lucina to attack them, in the hand to Special Summon, and in
// the opponent's Deck for them to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, HAND),
    (ID, HAND),
    (13597785, MZONE | OPPONENT),
    (ID, DECK | OPPONENT),
    (ID, DECK | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_CONTROL);
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
    e2.set_target(duel, |duel, a, chk, _| {
        super::c11677278::selfsptg_with(duel, a, chk, opponent_controls_more)
    });
    e2.set_operation(duel, |duel, a| super::c11677278::selfspop_with(duel, a, ID, opponent_controls_more));
    c.register_effect(duel, e2);
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    let c = a.e.get_handler(duel);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, 1 - tp, LOCATION_DECK as i32);
    if c.is_able_to_change_controler(duel) {
        duel.set_operation_info(0, CATEGORY_CONTROL, Some(c.into()), 1, tp, 0);
    }
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_MIMIGHOUL]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let p = 1 - tp;
    let mut break_chk = false;
    let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, p), p, LOCATION_DECK, 0, None);
    if !g.is_empty()
        && duel.get_location_count_full(p, LOCATION_MZONE, p, LOCATION_REASON_TOFIELD, 0xff) > 0
        && duel.select_yes_no(p, aux::stringid(ID, 2))?
    {
        duel.hint(HINT_SELECTMSG, p, HINTMSG_SPSUMMON);
        let sg = g.select(duel, p, 1, 1, None)?;
        break_chk = !sg.is_empty() && duel.special_summon(sg, 0, p, p, false, false, POS_FACEUP)? > 0;
    }
    super::c81522098::give_control(duel, e, tp, break_chk)
}

fn opponent_controls_more(duel: &Duel, tp: u8) -> bool {
    duel.get_field_group_count(tp, 0, LOCATION_MZONE) > duel.get_field_group_count(tp, LOCATION_MZONE, 0)
}
