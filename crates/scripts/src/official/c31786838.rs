// Regenesis
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 31786838;

pub const LISTED_SERIES: &[u16] = &[SET_REGENESIS];

// In the hand with Regenesis monsters in the Deck to take, and in the Graveyard with Regenesis monsters there to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (22938501, DECK),
    (96540807, DECK),
    (59323650, GRAVE),
    (22938501, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, aux::exccon);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, stock::special_summon_first_target::<POS_FACEUP>);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel, alt: bool) -> bool {
    filter1(c, duel) || (alt && filter2(c, duel))
}

fn filter1(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_REGENESIS]) && c.is_monster(duel) && (c.is_able_to_hand(duel) || c.is_able_to_grave(duel))
}

fn filter2(c: CardId, duel: &Duel) -> bool {
    c.is_attack(duel, &[2500]) && c.is_defense(duel, &[2500]) && c.is_able_to_hand(duel)
}

fn alt(duel: &Duel, tp: u8) -> bool {
    let filter = aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_REGENESIS]));
    duel.is_existing_matching_card(filter, tp, LOCATION_MZONE, 0, 1, None)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let alt = alt(duel, tp);
        return Ok(duel.is_existing_matching_card(|c, duel| thfilter(c, duel, alt), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let alt = alt(duel, tp);
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
    let filter = |c, duel: &Duel| thfilter(c, duel, alt);
    let Some(sc) = duel.select_matching_card(tp, filter, tp, LOCATION_DECK, 0, 1, 1, None)?.get_first() else {
        return Ok(());
    };
    if sc.is_set_card(duel, &[SET_REGENESIS]) {
        let to_grave = |duel: &mut Duel| duel.send_to_grave(sc, REASON_EFFECT).map(drop);
        aux::to_hand_or_else(duel, sc, tp, CardId::is_able_to_grave, to_grave, 574)?;
    } else {
        duel.send_to_hand(sc, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, sc)?;
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_REGENESIS]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let filter = |c, duel: &Duel| spfilter(c, duel, e, tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_target(filter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, filter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    Ok(false)
}
