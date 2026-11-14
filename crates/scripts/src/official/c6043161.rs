// R.B. Ga10 Driller
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 6043161;

pub const LISTED_SERIES: &[u16] = &[SET_RB];
// R.B. Link Monsters to link with, and the opponent's monsters to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (32216688, MZONE),
    (6821579, MZONE),
    (32216688, EXTRA),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, super::c33438265::spcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    e2.set_condition(duel, |duel, a| duel.is_main_phase() && super::c33438265::is_linked_with_rb(duel, a));
    e2.set_cost(duel, Cost::pay_lp::<500>);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let mut g = duel
        .select_target(tp, aux::TRUE, tp, 0, LOCATION_MZONE, 1, 1, None)?
        .unwrap_or_default();
    g.insert(a.e.get_handler(duel));
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 2, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && c.is_relate_to_effect(duel, a.e)
        && tc.is_relate_to_effect(duel, a.e)
        && tc.is_monster(duel)
    {
        duel.destroy(CardSet::from_iter([tc, c]), REASON_EFFECT)?;
    }
    Ok(())
}
