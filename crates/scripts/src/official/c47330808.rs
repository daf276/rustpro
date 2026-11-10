// Hapi, Guidance of Horus
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 47330808;

pub const LISTED_NAMES: &[u32] = &[CARD_KING_SARCOPHAGUS];

// In the Graveyard with King's Sarcophagus on the field to return, a copy on the field and in the Graveyard to target, and
// Raigeki and Dark Hole in the opponent's hand to destroy a card of ours.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, GRAVE),
    (ID, MZONE),
    (ID, GRAVE),
    (CARD_KING_SARCOPHAGUS, SZONE),
    (12580477, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, spcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_TODECK);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property_full(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e2.set_code(duel, EVENT_LEAVE_FIELD);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, thtdcon);
    e2.set_target(duel, thtdtg);
    e2.set_operation(duel, thtdop);
    c.register_effect(duel, e2);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    unported!(duel, !c.get_card_effect(duel, EFFECT_NECRO_VALLEY).is_empty());
    let tp = c.get_controler(duel);
    duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_code(duel, CARD_KING_SARCOPHAGUS)),
            tp,
            LOCATION_ONFIELD,
            0,
            1,
            None,
        )
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_previous_location(duel, LOCATION_ONFIELD)
        && c.is_previous_controler(duel, tp)
        && c.is_reason(duel, REASON_EFFECT)
        && c.get_reason_player(duel) == 1 - tp
}

fn thtdcon(duel: &Duel, a: &Args) -> bool {
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| cfilter(c, duel, a.tp), 1, None))
}

fn tgfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_can_be_effect_target(duel, e) && (c.is_able_to_hand(duel) || c.is_able_to_deck(duel))
}

fn rescon(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    let all = |f: fn(CardId, &Duel) -> bool| sg.filter_count(duel, f, None) == sg.len();
    (all(CardId::is_able_to_hand) || all(CardId::is_able_to_deck), false)
}

fn thtdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let g = duel.get_matching_group(
        |c, duel| tgfilter(c, duel, e),
        tp,
        LOCATION_GRAVE | LOCATION_REMOVED,
        LOCATION_GRAVE | LOCATION_REMOVED,
        None,
    );
    if chk == 0 {
        return Ok(g.len() >= 2 && aux::select_unselect_group_check(duel, g, 2, 2, &rescon));
    }
    let sg = aux::select_unselect_group(duel, g, 2, 2, &rescon, tp, HINTMSG_TARGET)?;
    duel.set_target_card(sg);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, Some(sg), 2, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_TODECK, Some(sg), 2, tp, 0);
    let rg = sg.filter(duel, |c, duel| c.is_location(duel, LOCATION_GRAVE), None);
    if !rg.is_empty() {
        duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(rg), rg.len() as u8, 0, 0);
    }
    Ok(false)
}

fn thtdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    if g.len() != 2 {
        return Ok(());
    }
    let b1 = g.filter_count(duel, CardId::is_able_to_hand, None) == 2;
    let b2 = g.filter_count(duel, CardId::is_able_to_deck, None) == 2;
    if !(b1 || b2) {
        return Ok(());
    }
    if duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])? == Some(1) {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    } else {
        duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    }
    Ok(())
}
