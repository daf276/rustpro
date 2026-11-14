// Tearlaments Metanoise
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 38436986;

pub const LISTED_NAMES: &[u32] = &[CARD_VISAS_STARFROST];
pub const LISTED_SERIES: &[u16] = &[SET_TEARLAMENTS];

// Set with Tearlaments Merrli on the field and in the Deck, Sangan to flip face-down, and in the hand for Tearlaments Heartbeat to send
// and the Deck for Merrli to mill, with Merrli in the Graveyard to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (ID, HAND),
    (ID, DECK),
    (ID, DECK),
    (74078255, MZONE),
    (74078255, MZONE),
    (74078255, HAND),
    (74078255, DECK),
    (74078255, GRAVE),
    (60362066, HAND),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_POSITION + CATEGORY_TOGRAVE + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(
        duel,
        TIMING_BATTLE_PHASE,
        TIMINGS_CHECK_MONSTER_E | TIMING_MAIN_END | TIMING_BATTLE_PHASE,
    );
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, poscon);
    e1.set_target(duel, postg);
    e1.set_operation(duel, posop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_reason(duel, REASON_EFFECT));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, stock::first_target_to_hand);
    c.register_effect(duel, e2);
}

pub fn visas_or_tearlaments(duel: &Duel, tp: u8) -> bool {
    let visas = aux::faceup_filter(|c, duel| c.is_code(duel, CARD_VISAS_STARFROST));
    let tearlaments = aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_TEARLAMENTS]));
    duel.is_existing_matching_card(visas, tp, LOCATION_ONFIELD, 0, 1, None)
        || duel.is_existing_matching_card(tearlaments, tp, LOCATION_MZONE, 0, 1, None)
}

fn poscon(duel: &Duel, a: &Args) -> bool {
    visas_or_tearlaments(duel, a.tp)
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_TEARLAMENTS]) && c.is_monster(duel) && c.is_able_to_grave(duel)
}

fn postg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp)
            && chkc.is_location(duel, LOCATION_MZONE)
            && chkc.is_faceup(duel)
            && chkc.is_can_turn_set(duel));
    }
    let filter = aux::faceup_filter(CardId::is_can_turn_set);
    if chk == 0 {
        return Ok(duel.is_existing_target(&filter, tp, 0, LOCATION_MZONE, 1, None)
            && duel.is_existing_matching_card(tgfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
    let g = duel.select_target(tp, filter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_POSITION, g, 1, tp, POS_FACEDOWN_DEFENSE as i32);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, a.e) || tc.is_facedown(duel) || duel.change_position(tc, POS_FACEDOWN_DEFENSE)? < 1 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, tgfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.break_effect()?;
        duel.send_to_grave(g, REASON_EFFECT)?;
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_TEARLAMENTS]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && thfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, 0, 0);
    Ok(false)
}
