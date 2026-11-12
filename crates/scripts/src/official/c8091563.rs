// Super Defense Robot Lio
use crate::prelude::*;
use crate::testing::{Check, HAND};

const ID: u32 = 8091563;

pub const LISTED_NAMES: &[u32] = &[71071546];

// Copies in the hand to Special Summon after a Normal Summon. The Graveyard return needs a Super Defense Robot sent from there to the
// hand, which only the unported Super Defense Robot Monki does.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (ID, HAND), (ID, HAND)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_target(duel, sumtg);
    e1.set_operation(duel, sumop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EVENT_TO_HAND);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, stock::special_summon_first_target::<POS_FACEUP>);
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    (c.is_set_card(duel, &[SET_SUPER_DEFENSE_ROBOT]) || c.is_code(duel, 71071546))
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sumtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| filter(c, duel, a.e, a.tp), LOCATION_HAND)
}

fn sumop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| filter(c, duel, a.e, a.tp), LOCATION_HAND, POS_FACEUP)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let (tp, eg) = (a.tp, a.eg.unwrap_or_default());
    let Some(tc) = eg.get_first() else { return false };
    eg.len() == 1
        && (tc.is_set_card(duel, &[SET_SUPER_DEFENSE_ROBOT]) || tc.is_code(duel, 71071546))
        && tc.is_controler(duel, tp)
        && tc.is_previous_controler(duel, tp)
        && tc.is_previous_location(duel, LOCATION_GRAVE)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp, eg) = (a.e, a.tp, a.eg.unwrap_or_default());
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && eg
                .get_first()
                .is_some_and(|tc| tc.is_can_be_special_summoned(duel, e, 0, tp, false, false)));
    }
    duel.set_target_card(eg);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(eg), 1, 0, 0);
    Ok(false)
}
