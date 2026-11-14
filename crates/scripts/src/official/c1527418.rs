// Wiz, Sage Fur Hire
use super::c38916526::{negcost, negtg};
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 1527418;

pub const LISTED_SERIES: &[u16] = &[SET_FUR_HIRE];
pub const LISTED_NAMES: &[u32] = &[ID];
// Revived by Mayhem Fur Hire beside other Fur Hire monsters, with Fur Hire monsters in the hand to discard against the
// opponent's Pot of Greed.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, GRAVE),
    (91405870, HAND),
    (93850652, MZONE),
    (47647354, MZONE),
    (93850652, HAND),
    (28004531, HAND),
    (55144522, HAND | OPPONENT),
    (55144522, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_RECOVER);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, rectg);
    e1.set_operation(duel, recop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_NEGATE + CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, negcon);
    e2.set_cost(duel, |duel, a, chk| Cost::replaceable(duel, a, chk, negcost));
    e2.set_target(duel, negtg);
    e2.set_operation(duel, |duel, a| {
        duel.negate_activation(a.ev as u8);
        Ok(())
    });
    c.register_effect(duel, e2);
    aux::double_snare_validity(duel, c, LOCATION_MZONE);
}

fn recfilter(c: CardId, duel: &Duel) -> bool {
    !c.is_code(duel, ID) && c.is_faceup(duel) && c.is_set_card(duel, &[SET_FUR_HIRE])
}

fn recover_amount(duel: &Duel, tp: u8) -> u32 {
    let g = duel.get_matching_group(recfilter, tp, LOCATION_MZONE, 0, None);
    g.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) as u32 * 500
}

fn rectg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(recfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    let rec = recover_amount(duel, tp);
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, 1 - tp, rec as i32);
    Ok(false)
}

fn recop(duel: &mut Duel, a: &Args) -> Script<()> {
    let rec = recover_amount(duel, a.tp);
    duel.recover(a.tp, rec, REASON_EFFECT)?;
    Ok(())
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED)
        && a.ep == 1 - a.tp
        && a.re.is_some_and(|re| re.is_spell_trap_effect(duel))
        && duel.is_chain_negatable(a.ev as u8)
}
