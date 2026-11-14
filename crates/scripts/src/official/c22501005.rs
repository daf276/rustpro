// Ritual Beast Inheritance
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, REMOVED, SZONE};

const ID: u32 = 22501005;

pub const LISTED_SERIES: &[u16] = &[SET_RITUAL_BEAST];
// On the field, with Ritual Beasts of several Types in the hand and Deck to reveal and search, and Ritual Beast Ulti-Apelio
// on the field to Special Summon two banished Ritual Beasts at once.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (14513016, HAND),
    (88123329, HAND),
    (14513016, DECK),
    (88123329, DECK),
    (13890468, DECK),
    (86274272, MZONE),
    (14513016, REMOVED),
    (88123329, REMOVED),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_target_range(duel, 0, LOCATION_MZONE);
    e2.set_value(duel, Value::Card(atkvalue));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_HANDES);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_cost(duel, thcost);
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_POSITION);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e4.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_count_limit(duel, 1, (ID, 1));
    e4.set_condition(duel, |duel, a| {
        a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| c.is_controler(duel, a.tp), 2, None))
    });
    e4.set_target(duel, postg);
    e4.set_operation(duel, posop);
    c.register_effect(duel, e4);
}

fn atkvalue(duel: &Duel, e: EffectId, _: CardId) -> i32 {
    let filter = aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_RITUAL_BEAST]));
    let g = duel.get_matching_group(filter, e.get_handler_player(duel), LOCATION_MZONE, 0, None);
    -200 * g.get_class_count(duel, |c, duel| c.get_race(duel) as i64) as i32
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_RITUAL_BEAST])
        && c.is_monster(duel)
        && !c.is_public(duel)
        && duel.is_existing_matching_card(|tc, duel| thfilter(tc, duel, c.get_race(duel)), tp, LOCATION_DECK, 0, 1, None)
}

fn thfilter(c: CardId, duel: &Duel, rac: u64) -> bool {
    c.is_set_card(duel, &[SET_RITUAL_BEAST]) && c.is_monster(duel) && c.is_able_to_hand(duel) && c.is_race_except(duel, rac)
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let filter = |c, duel: &Duel| cfilter(c, duel, tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
    let rc = duel.select_matching_card(tp, filter, tp, LOCATION_HAND, 0, 1, 1, None)?;
    duel.confirm_cards(1 - tp, rc)?;
    duel.shuffle_hand(tp)?;
    a.e.set_label_object_group(duel, Some(rc));
    duel.set_target_card(rc);
    Ok(false)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, a.tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, a.tp, 1);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(rc) = e.get_label_object_group(duel).and_then(|g| g.get_first()) else {
        return Ok(());
    };
    if !rc.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let race = rc.get_race(duel);
    let g = duel.select_matching_card(tp, |c, duel| thfilter(c, duel, race), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty()
        && duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? > 0
        && g.get_first().is_some_and(|c| c.is_location(duel, LOCATION_HAND))
    {
        duel.confirm_cards(1 - tp, g)?;
        duel.shuffle_hand(tp)?;
        duel.break_effect()?;
        duel.discard_hand(tp, aux::TRUE, 1, 1, REASON_EFFECT | REASON_DISCARD)?;
    }
    Ok(())
}

fn postg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_can_change_position(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_can_change_position, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
    let g = duel.select_target(tp, CardId::is_can_change_position, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_POSITION, g, 1, 0, 0);
    Ok(false)
}

fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.change_position_full(
            tc,
            [POS_FACEUP_DEFENSE, POS_FACEUP_ATTACK, POS_FACEUP_ATTACK, POS_FACEUP_ATTACK],
            false,
        )?;
    }
    Ok(())
}
