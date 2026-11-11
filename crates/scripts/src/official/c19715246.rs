// Cyber Jormungardr
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 19715246;

pub const LISTED_NAMES: &[u32] = &[CARD_CYBER_DRAGON, CARD_POLYMERIZATION];
// In the hand and on the field with Cyber Dragon in the Deck and on the field, Polymerization in the Deck and Graveyard,
// and a monster of the opponent's.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (CARD_CYBER_DRAGON, DECK),
    (CARD_CYBER_DRAGON, DECK),
    (CARD_CYBER_DRAGON, MZONE),
    (CARD_POLYMERIZATION, DECK),
    (CARD_POLYMERIZATION, GRAVE),
    (CARD_CYBER_DRAGON, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_unsummonable(duel);
    c.add_must_be_special_summoned_by_card_effect(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_EQUIP);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| duel.get_field_group_count(a.tp, 0, LOCATION_MZONE) > 0);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, thcost);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, freezone: i32) -> bool {
    c.is_can_be_special_summoned(duel, e, 0, tp, false, false) && freezone > 0
}

fn eqpfilter(c: CardId, duel: &Duel, tp: u8, freezone: i32) -> bool {
    c.check_unique_on_field(duel, tp) && !c.is_forbidden(duel) && freezone > 0
}

fn cyberdragonfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mon_ft: i32, st_ft: i32) -> bool {
    c.is_code(duel, CARD_CYBER_DRAGON) && (spfilter(c, duel, e, tp, mon_ft) || eqpfilter(c, duel, tp, st_ft))
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let mon_ft = duel.get_location_count(tp, LOCATION_MZONE);
    let st_ft = duel.get_location_count(tp, LOCATION_SZONE);
    if chk == 0 {
        return Ok(mon_ft > 0
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && duel.is_existing_matching_card(
                |c, duel| cyberdragonfilter(c, duel, e, tp, mon_ft - 1, st_ft),
                tp,
                LOCATION_DECK,
                0,
                1,
                None,
            ));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_EQUIP, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        let mon_ft = duel.get_location_count(tp, LOCATION_MZONE);
        let st_ft = duel.get_location_count(tp, LOCATION_SZONE);
        duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
        let g = duel.select_matching_card(
            tp,
            |c, duel| cyberdragonfilter(c, duel, e, tp, mon_ft, st_ft),
            tp,
            LOCATION_DECK,
            0,
            1,
            1,
            None,
        )?;
        if let Some(tc) = g.get_first() {
            let b1 = spfilter(tc, duel, e, tp, mon_ft);
            let b2 = eqpfilter(tc, duel, tp, st_ft);
            let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 3)), (b2, aux::stringid(ID, 4))])?;
            duel.break_effect()?;
            if op == Some(1) {
                duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
            } else if op == Some(2) && duel.equip(tp, tc, c)? {
                let e0 = duel.create_effect(tc);
                e0.set_type(duel, EFFECT_TYPE_SINGLE);
                e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
                e0.set_code(duel, EFFECT_EQUIP_LIMIT);
                e0.set_value(duel, Value::Card(super::c38423248::equip_limit));
                e0.set_label_object_group(duel, Some(c.into()));
                e0.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
                tc.register_effect(duel, e0);
            }
        }
    }
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 5));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| !c.is_race(duel, RACE_MACHINE));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}

fn thcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_CYBER_DRAGON) && c.is_faceup(duel) && c.is_able_to_hand_as_cost(duel)
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thcostfilter, tp, LOCATION_ONFIELD, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_matching_card(tp, thcostfilter, tp, LOCATION_ONFIELD, 0, 1, 1, None)?;
    duel.send_to_hand(g, PLAYER_NONE, REASON_COST)?;
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_POLYMERIZATION) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK | LOCATION_GRAVE, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(thfilter),
        tp,
        LOCATION_DECK | LOCATION_GRAVE,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
