// Salamangreat Circle
use crate::prelude::*;
use crate::testing::{DECK, HAND};

const ID: u32 = 52155219;

pub const LISTED_SERIES: &[u16] = &[SET_SALAMANGREAT];
// Salamangreat Gazelles in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (26889158, DECK), (26889158, DECK), (26889158, DECK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_END_PHASE);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && immfilter(chkc, duel));
    }
    let b1 = duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None);
    let b2 = duel.is_existing_target(immfilter, tp, LOCATION_MZONE, 0, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 0)), (b2, aux::stringid(ID, 1))])?;
    if op == Some(1) {
        e.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
        e.set_property(duel, 0);
        e.set_operation(duel, thop);
    } else {
        e.set_category(duel, 0);
        e.set_property(duel, EFFECT_FLAG_CARD_TARGET);
        e.set_operation(duel, immop);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_APPLYTO);
        duel.select_target(tp, immfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    }
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SALAMANGREAT]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn immfilter(c: CardId, duel: &Duel) -> bool {
    c.is_link_monster(duel) && c.is_set_card(duel, &[SET_SALAMANGREAT]) && c.is_reincarnation_summoned(duel)
}

fn immop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_description(duel, 3101);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_range(duel, LOCATION_MZONE);
        e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
        e1.set_value(
            duel,
            Value::Effect(|duel, e, te| te.is_monster_effect(duel) && te.get_owner(duel) != e.get_owner(duel)),
        );
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
