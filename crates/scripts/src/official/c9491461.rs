// Gagaga Ganbara Knight
use crate::prelude::*;
use crate::testing::{Check, HAND};

const ID: u32 = 9491461;

pub const LISTED_SERIES: &[u16] = &[SET_GAGAGA, SET_GOGOGO];
// Every effect needs a Gagaga monster in the Extra Deck to reveal or an Xyz Monster to detach this card from, and no Gagaga Xyz Monster is ported.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND)];
pub const CHECKS: &[Check] = &[
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0))),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_POSITION);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, spcost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, thcon);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_REMOVE);
    c.register_effect(duel, e3);
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let filter: fn(CardId, &Duel, EffectId, u8) -> bool = |c, duel, _, _| c.is_set_card(duel, &[SET_GAGAGA]);
    Cost::reveal(duel, a, chk, filter, false, (1, 1), None, LOCATION_EXTRA)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    duel.set_possible_operation_info(0, CATEGORY_POSITION, None, 1, PLAYER_EITHER, LOCATION_MZONE as i32);
    Ok(false)
}

fn gaspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_GAGAGA]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !(c.is_relate_to_effect(duel, e) && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0) {
        return Ok(());
    }
    let b1 = duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(|c, duel| gaspfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, None);
    let b2 = duel.is_existing_matching_card(CardId::is_can_change_position, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None);
    if !((b1 || b2) && duel.select_yes_no(tp, aux::stringid(ID, 2))?) {
        return Ok(());
    }
    match duel.select_effect(tp, &[(b1, aux::stringid(ID, 3)), (b2, aux::stringid(ID, 4))])? {
        Some(1) => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let g = duel.select_matching_card(tp, |c, duel| gaspfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
            if !g.is_empty() {
                duel.break_effect()?;
                duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
            }
        }
        Some(2) => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
            let g = duel.select_matching_card(tp, CardId::is_can_change_position, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
            if !g.is_empty() {
                duel.hint_selection(g);
                duel.break_effect()?;
                duel.change_position_full(
                    g,
                    [POS_FACEUP_DEFENSE, POS_FACEUP_DEFENSE, POS_FACEUP_ATTACK, POS_FACEUP_ATTACK],
                    false,
                )?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_reason(duel, REASON_COST)
        && a.re.is_some_and(|re| re.is_activated(duel) && re.is_active_type(duel, TYPE_XYZ))
        && c.is_previous_location(duel, LOCATION_OVERLAY)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GOGOGO]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
