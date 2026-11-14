// Gem-Knight Hollowcore
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 88225269;

pub const LISTED_SERIES: &[u16] = &[SET_GEM_KNIGHT];
pub const LISTED_NAMES: &[u32] = &[GEM_KNIGHT_FUSION];

const GEM_KNIGHT_FUSION: u32 = 1264319;

// Gem-Knight Fusion and Garnet in the Deck to send for the summon; in the GY with two Garnet beside it and Gem-Knight Zirconia on
// the field for the negation.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (GEM_KNIGHT_FUSION, DECK),
    (91731841, DECK),
    (91731841, GRAVE),
    (91731841, GRAVE),
    (8692301, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, spcost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP_DEFENSE>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DISABLE + CATEGORY_ATKCHANGE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, negcon);
    e2.set_cost(duel, negcost);
    e2.set_target(duel, negtg);
    e2.set_operation(duel, negop);
    c.register_effect(duel, e2);
}

fn spcostfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_code(duel, GEM_KNIGHT_FUSION) || (c.is_set_card(duel, &[SET_GEM_KNIGHT]) && c.is_type(duel, TYPE_NORMAL)))
        && c.is_able_to_grave_as_cost(duel)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::send_to_grave_cost(duel, a, chk, spcostfilter, LOCATION_DECK)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn gemfusfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GEM_KNIGHT]) && c.is_type(duel, TYPE_FUSION) && c.is_faceup(duel)
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    a.ep == 1 - a.tp
        && duel.is_chain_disablable(a.ev as u8)
        && duel.is_existing_matching_card(gemfusfilter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn negcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GEM_KNIGHT]) && c.is_able_to_remove_as_cost(duel)
}

fn negcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(
            c.is_able_to_remove_as_cost(duel) && duel.is_existing_matching_card(negcostfilter, tp, LOCATION_GRAVE, 0, 2, Some(c.into()))
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, negcostfilter, tp, LOCATION_GRAVE, 0, 2, 2, Some(c.into()))?;
    duel.remove(g.union(c.into()), POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn gem_knights(duel: &Duel, tp: u8) -> CardSet {
    duel.get_matching_group(
        aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_GEM_KNIGHT])),
        tp,
        LOCATION_MZONE,
        0,
        None,
    )
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, 0, 0);
    let g = gem_knights(duel, a.tp);
    duel.set_operation_info(0, CATEGORY_ATKCHANGE, Some(g), g.len() as u8, a.tp, 1000);
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.negate_effect(a.ev as u8) {
        let g = gem_knights(duel, a.tp);
        if g.is_empty() {
            return Ok(());
        }
        let c = a.e.get_handler(duel);
        duel.break_effect()?;
        for tc in g {
            tc.update_attack(duel, 1000, Some(RESET_EVENT | RESETS_STANDARD), Some(c), None);
        }
    }
    Ok(())
}
