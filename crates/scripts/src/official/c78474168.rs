// Breakthrough Skill
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 78474168;

// In the hand to Activate and in the Graveyard against the opponent's effect monsters: Sangan and Elfnote Lucina.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (26202165, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER);
    e1.set_target(duel, target);
    e1.set_operation(duel, |duel, a| activate(duel, a, false));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DISABLE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_condition(duel, |duel, a| aux::exccon(duel, a) && duel.is_turn_player(a.tp));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, target);
    e2.set_operation(duel, |duel, a| activate(duel, a, true));
    c.register_effect(duel, e2);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_EFFECT) && !c.is_disabled(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, filter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    Ok(false)
}

// The Graveyard effect also rechecks that the target is an Effect Monster.
fn activate(duel: &mut Duel, a: &Args, recheck_effect: bool) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e)
        && tc.is_faceup(duel)
        && !tc.is_disabled(duel)
        && tc.is_controler(duel, 1 - tp)
        && (!recheck_effect || tc.is_type(duel, TYPE_EFFECT))
    {
        duel.negate_related_chain(tc, RESET_TURN_SET);
        super::c97268402::negate(duel, c, tc, 0);
    }
    Ok(())
}
