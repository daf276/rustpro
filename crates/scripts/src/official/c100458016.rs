// Cielo, the Veil of Raise Moon
// The other Raise Moon cards share the functions generic over the card's id.
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 100458016;

// Copies to draw, to Special Summon from the hand and to activate on the field.
pub const SUPPORT: &[(u32, u16)] = &[(ID, DECK), (ID, DECK), (ID, DECK), (ID, HAND), (ID, MZONE)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = register_draw_summon::<0>(duel, c, ID);
    let e2 = e1.clone_effect(duel);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_cost(duel, selfspcost);
    e2.set_target(duel, |duel, a, chk, _| {
        let c = a.e.get_handler(duel);
        if chk == 0 {
            return Ok(c.is_can_be_special_summoned(duel, a.e, 0, a.tp, false, false));
        }
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, a.tp, 0);
        Ok(false)
    });
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_DRAW);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_condition(duel, |duel, _| duel.is_main_phase());
    e3.set_cost(duel, drcost::<ID, 3, { RESET_PHASE | PHASE_END }>);
    e3.set_target(duel, draw_target);
    e3.set_operation(duel, draw);
    e3.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    c.register_effect(duel, e3);
    track_searches::<ID, { RESET_PHASE | PHASE_END }>(duel, c);
}

// "When you draw this card: You can reveal it; Special Summon it", with the summon type the card passes.
pub(crate) fn register_draw_summon<const SUMTYPE: u32>(duel: &mut Duel, c: CardId, id: u32) -> EffectId {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(id, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_DRAW);
    e1.set_cost(duel, Cost::self_reveal);
    e1.set_target(duel, draw_summon_target::<SUMTYPE>);
    e1.set_operation(duel, draw_summon::<SUMTYPE>);
    c.register_effect(duel, e1);
    e1
}

fn draw_summon_target<const SUMTYPE: u32>(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(
            duel.get_location_count(a.tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, a.e, SUMTYPE, a.tp, false, false)
        );
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, a.tp, 0);
    Ok(false)
}

fn draw_summon<const SUMTYPE: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.special_summon(c, SUMTYPE, a.tp, a.tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

// The global check flagging a player who added cards from the Deck to the hand except by drawing.
pub(crate) fn track_searches<const ID: u32, const RESET: u32>(duel: &mut Duel, c: CardId) {
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_TO_HAND);
        ge1.set_promptless_operation(duel, flag_searches::<ID, RESET>);
        duel.register_effect(ge1, 0);
    });
}

fn flag_searches<const ID: u32, const RESET: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
    for ec in a.eg.unwrap_or_default() {
        if ec.is_previous_location(duel, LOCATION_DECK) && !ec.is_reason(duel, REASON_DRAW) {
            duel.register_flag_effect(ec.get_reason_player(duel), ID, RESET, 0, 1);
        }
    }
    Ok(())
}

fn selfspcostfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    duel.get_mzone_count(tp, Some(c.into())) > 0
        && (c.is_location(duel, LOCATION_HAND) || c.is_faceup(duel))
        && c.is_able_to_deck_as_cost(duel)
}

fn selfspcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let filter = |sc: CardId, duel: &Duel| selfspcostfilter(sc, duel, tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_HAND | LOCATION_ONFIELD, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_ONFIELD, 0, 1, 1, Some(c.into()))?;
    let Some(sc) = g.get_first() else { return Ok(false) };
    if sc.is_on_field(duel) {
        duel.hint_selection(sc);
    }
    duel.send_to_deck(sc, PLAYER_NONE, SEQ_DECKBOTTOM, REASON_COST)?;
    Ok(false)
}

// "You cannot add cards from the Deck to the hand ... except by drawing them", for the turn or (RESET 0) the Duel.
pub(crate) fn drcost<const ID: u32, const DESCRIPTION: u64, const RESET: u32>(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(!duel.has_flag_effect(a.tp, ID));
    }
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, DESCRIPTION));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_TO_HAND);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_location(duel, LOCATION_DECK));
    if RESET != 0 {
        e1.set_reset(duel, RESET);
    }
    duel.register_effect(e1, a.tp);
    Ok(false)
}

pub(crate) fn draw_target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_player_can_draw(a.tp, 1));
    }
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, a.tp, 1);
    Ok(false)
}

pub(crate) fn draw(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.draw(a.tp, 1, REASON_EFFECT)?;
    Ok(())
}
