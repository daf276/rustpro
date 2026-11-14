// Brightest, Blazing, Branded King
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 19271881;

pub const LISTED_NAMES: &[u32] = &[CARD_ALBAZ];
// Albion face-up to enable, Fallen of Albaz with Branded Fusion to send Albion to the Graveyard, and opponent's cards to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, GRAVE),
    (87746184, MZONE),
    (87746184, EXTRA),
    (87746184, EXTRA),
    (CARD_ALBAZ, HAND),
    (44362883, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, distg);
    e1.set_operation(duel, disop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_hint_timing(duel, TIMING_END_PHASE, TIMING_END_PHASE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| duel.has_flag_effect(a.tp, ID) && duel.is_phase(PHASE_END));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_TO_GRAVE);
        ge1.set_operation(duel, checkop);
        duel.register_effect(ge1, 0);
    });
}

fn checkop(duel: &mut Duel, a: &Args) -> Script<()> {
    for tc in a.eg.unwrap_or_default() {
        if tc.is_type(duel, TYPE_FUSION) {
            let controler = tc.get_controler(duel);
            duel.register_flag_effect(controler, ID, RESET_PHASE | PHASE_END, 0, 1);
        }
    }
    Ok(())
}

fn cfilter(c: CardId, duel: &Duel, sc: CardId) -> bool {
    c.is_faceup(duel)
        && c.is_type(duel, TYPE_FUSION)
        && c.lists_code_as_material(duel, &[CARD_ALBAZ])
        && duel.is_existing_matching_card(
            CardId::is_negatable,
            0,
            LOCATION_ONFIELD,
            LOCATION_ONFIELD,
            1,
            Some(CardSet::from_iter([c, sc])),
        )
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|tc, duel| cfilter(tc, duel, c), tp, LOCATION_MZONE, 0, 1, None));
    }
    let g = duel.get_matching_group(CardId::is_negatable, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, Some(c.into()));
    duel.set_operation_info(0, CATEGORY_DISABLE, Some(g), (g.len() as u8).wrapping_sub(1), 0, 0);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    let mut exc = duel.select_matching_card(tp, |tc, duel| cfilter(tc, duel, c), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    if exc.is_empty() {
        return Ok(());
    }
    duel.hint_selection_full(exc, true);
    exc.insert(c);
    let g = duel.get_matching_group(CardId::is_negatable, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, Some(exc));
    for tc in g {
        duel.negate_related_chain(tc, RESET_TURN_SET);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_DISABLE);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_DISABLE_EFFECT);
        e2.set_value(duel, RESET_TURN_SET as i32);
        tc.register_effect(duel, e2);
    }
    Ok(())
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.send_to_hand(c, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - a.tp, c)?;
    }
    Ok(())
}
