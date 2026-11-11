// PSY-Framegear Alpha
use super::c1697104::summon_with_driver;
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, OPPONENT};

const ID: u32 = 75425043;

pub const LISTED_NAMES: &[u32] = &[CARD_PSYFRAME_DRIVER];

// In the hand with PSY-Frame Driver and PSY-Framegear Gamma in the Deck, against monsters the opponent Summons.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (CARD_PSYFRAME_DRIVER, DECK),
    (CARD_PSYFRAME_DRIVER, DECK),
    (38814750, DECK),
    (38814750, DECK),
    (26202165, HAND | OPPONENT),
    (26202165, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_unsummonable(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e1.set_value(
        duel,
        Value::SpSummonCondition(|duel, _, se, _, _, _, _| se.is_some_and(|se| se.is_has_type(duel, EFFECT_TYPE_ACTIONS))),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_condition(duel, condition);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.unwrap_or_default()
        .is_exists(duel, |c, duel| c.is_summon_player(duel, 1 - tp), 1, None)
        && (duel.get_field_group_count(tp, LOCATION_MZONE, 0) == 0 || duel.is_player_affected_by_effect(tp, CARD_PSYFRAME_LAMBDA).is_some())
}

fn spfilter1(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, CARD_PSYFRAME_DRIVER)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, Some(c.into()))
}

fn spfilter2(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, CARD_PSYFRAME_DRIVER)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && duel.is_existing_matching_card(thfilter0, tp, LOCATION_DECK, 0, 1, Some(c.into()))
}

fn thfilter0(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_PSY_FRAME]) && !c.is_code(duel, ID)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    thfilter0(c, duel) && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && duel.get_location_count(tp, LOCATION_MZONE) > 1
            && e.get_handler(duel).is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && duel.is_existing_matching_card(
                |c, duel| spfilter1(c, duel, e, tp),
                tp,
                LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE,
                0,
                1,
                None,
            ));
    }
    duel.set_operation_info(
        0,
        CATEGORY_SPECIAL_SUMMON,
        None,
        2,
        tp,
        (LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE) as i32,
    );
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    if summon_with_driver::<ID>(duel, a, spfilter2)? {
        let tp = a.tp;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let g2 = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g2.is_empty() {
            duel.send_to_hand(g2, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, g2)?;
        }
    }
    Ok(())
}
