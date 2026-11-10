// Nadir Servant
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND};

const ID: u32 = 1984618;

pub const LISTED_NAMES: &[u32] = &[CARD_ALBAZ];
// Fallen of Albaz, in the Deck, to search with an Extra Deck monster of at least its ATK, Secreterion Dragon, to send.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (89851827, EXTRA), (CARD_ALBAZ, DECK), (CARD_ALBAZ, DECK)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn thfilter(c: CardId, duel: &Duel, atk: i32) -> bool {
    (c.is_code(duel, CARD_ALBAZ) || (c.is_monster(duel) && c.is_set_card(duel, &[SET_DOGMATIKA])))
        && c.get_attack(duel) <= atk
        && c.is_able_to_hand(duel)
}

fn tgfilter(c: CardId, duel: &Duel, tp: u8, necro: bool) -> bool {
    let atk = c.get_attack(duel);
    c.is_able_to_grave(duel)
        && if necro {
            duel.is_existing_matching_card(
                aux::necro_valley_filter(|c, duel| thfilter(c, duel, atk)),
                tp,
                LOCATION_DECK | LOCATION_GRAVE,
                0,
                1,
                None,
            )
        } else {
            duel.is_existing_matching_card(|c, duel| thfilter(c, duel, atk), tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, None)
        }
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| tgfilter(c, duel, tp, false), tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_EXTRA as i32);
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, (LOCATION_DECK | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g1 = duel.select_matching_card(tp, |c, duel| tgfilter(c, duel, tp, true), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if !g1.is_empty() && duel.send_to_grave(g1, REASON_EFFECT)? > 0 {
        let og = duel.get_operated_group();
        if og.get_first().is_some_and(|c| c.is_location(duel, LOCATION_GRAVE)) {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
            let atk = g1.get_first().map_or(0, |c| c.get_attack(duel));
            let g2 = duel.select_matching_card(
                tp,
                aux::necro_valley_filter(|c, duel| thfilter(c, duel, atk)),
                tp,
                LOCATION_DECK | LOCATION_GRAVE,
                0,
                1,
                1,
                None,
            )?;
            if !g2.is_empty() {
                duel.break_effect()?;
                duel.send_to_hand(g2, PLAYER_NONE, REASON_EFFECT)?;
                duel.confirm_cards(1 - tp, g2)?;
            }
        }
    }
    if !e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        return Ok(());
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_location(duel, LOCATION_EXTRA));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::register_client_hint(duel, e.get_handler(duel), EFFECT_FLAG_OATH, tp, 1, 0, Some(aux::stringid(ID, 1)));
    Ok(())
}
