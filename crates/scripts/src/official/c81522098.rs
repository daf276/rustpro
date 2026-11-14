// Mimighoul Dragon
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 81522098;

pub const LISTED_SERIES: &[u16] = &[SET_MIMIGHOUL];
// Copies face-down on the field to Flip with the opponent's Elfnote Lucina to attack them, in the hand to Special Summon, and
// Mimighoul Room in the Deck to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, HAND),
    (13597785, MZONE | OPPONENT),
    (59293853, DECK),
    (59293853, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_CONTROL);
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
    e2.set_target(duel, selfsptg);
    e2.set_operation(duel, selfspop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_SUMMON_SUCCESS);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e4);
}

fn desfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && !c.is_set_card(duel, &[SET_MIMIGHOUL])
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    let dg = duel.get_matching_group(desfilter, tp, LOCATION_MZONE, 0, None);
    if !dg.is_empty() {
        duel.set_operation_info(0, CATEGORY_DESTROY, Some(dg), dg.len() as u8, tp, 0);
    }
    let c = a.e.get_handler(duel);
    if c.is_able_to_change_controler(duel) {
        duel.set_operation_info(0, CATEGORY_CONTROL, Some(c.into()), 1, tp, 0);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mut break_chk = false;
    let dg = duel.get_matching_group(desfilter, tp, LOCATION_MZONE, 0, None);
    if !dg.is_empty() {
        duel.destroy(dg, REASON_EFFECT)?;
        break_chk = true;
    }
    give_control(duel, e, tp, break_chk)
}

// The last step of the Mimighoul Flip effects: control of the card goes to the opponent.
pub fn give_control(duel: &mut Duel, e: EffectId, tp: u8, break_chk: bool) -> Script<()> {
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) {
        if break_chk {
            duel.break_effect()?;
        }
        duel.get_control(c, 1 - tp)?;
    }
    Ok(())
}

pub fn selfsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(super::c11677278::can_set_to_opponent(duel, a.e, a.tp));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, a.tp, 0);
    Ok(false)
}

pub fn selfspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.special_summon(c, 0, tp, 1 - tp, false, false, POS_FACEDOWN_DEFENSE)?;
        duel.confirm_cards(tp, c)?;
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MIMIGHOUL]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
