// Traptrix Mantis
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, SZONE};

const ID: u32 = 82738277;

pub const LISTED_SERIES: &[u16] = &[SET_TRAP_HOLE, SET_HOLE, SET_TRAPTRIX];
// Copies in the hand and on the field, Traptrix Myrmeleo to search, and Set Trap Holes to return.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (91812341, DECK),
    (91812341, DECK),
    (4206964, SZONE),
    (4206964, SZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e1.set_value(duel, Value::Effect(super::c91812341::efilter));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_TRIGGER_O + EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SET);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_count_limit(duel, 1, 0);
    e3.set_hint_timing(duel, 0, TIMING_END_PHASE);
    e3.set_target(duel, rettg);
    e3.set_operation(duel, retop);
    c.register_effect(duel, e3);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_set_card(duel, &[SET_TRAPTRIX]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, filter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, filter, LOCATION_DECK)
}

fn retfilter(c: CardId, duel: &Duel) -> bool {
    c.is_facedown(duel) && c.is_able_to_hand(duel)
}

fn rettg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_SZONE) && chkc.is_controler(duel, tp) && retfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(retfilter, tp, LOCATION_SZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_target(tp, retfilter, tp, LOCATION_SZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, 0, 0);
    Ok(false)
}

fn retop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !(tc.is_relate_to_effect(duel, a.e)
        && tc.is_facedown(duel)
        && duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)? > 0
        && tc.is_location(duel, LOCATION_HAND))
    {
        return Ok(());
    }
    let g = duel.get_matching_group(CardId::is_ssetable, tp, LOCATION_HAND, 0, None);
    if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        duel.shuffle_hand(tp)?;
        duel.break_effect()?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
        let sg = g.select(duel, tp, 1, 1, None)?;
        if let Some(sc) = sg.get_first() {
            duel.sset_full(tp, sc, tp, false)?;
        }
    }
    Ok(())
}
