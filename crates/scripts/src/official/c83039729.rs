// Grandmaster of the Six Samurai
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 83039729;

pub const LISTED_SERIES: &[u16] = &[SET_SIX_SAMURAI];
// In the hand with Six Samurai (Kageki) on the field to summon it, others in the Graveyard to add, and the opponent's Raigeki
// to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (2511717, MZONE),
    (2511717, MZONE),
    (2511717, GRAVE),
    (44686185, GRAVE),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.set_unique_on_field(duel, 1, 0, |c, duel| c.is_code(duel, ID), LOCATION_ONFIELD);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, spcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_condition(duel, thcon);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(
            |c, duel| c.is_faceup(duel) && c.is_set_card(duel, &[SET_SIX_SAMURAI]),
            tp,
            LOCATION_MZONE,
            0,
            1,
            None,
        )
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    !c.is_reason(duel, REASON_BATTLE) && a.rp == 1 - a.tp && c.is_previous_controler(duel, a.tp)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SIX_SAMURAI]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    targets_to_hand(duel, a, chk, chkc, filter)
}

// The target of Grandmaster and Shinai, which differ in their filter.
pub fn targets_to_hand(
    duel: &mut Duel,
    a: &Args,
    chk: i32,
    chkc: Option<CardId>,
    filter: impl Fn(CardId, &Duel) -> bool + Copy,
) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, filter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, 0, 0);
    Ok(false)
}

pub fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - a.tp, tc)?;
    }
    Ok(())
}
