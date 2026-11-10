// Time Thief Redoer
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 55285840;

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

// In the Extra Deck, Xyz Summoned with two Level 4 monsters on the field, with a monster of the opponent's to send to the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (9491461, MZONE),
    (9491461, MZONE),
    (23720856, MZONE),
    (23720856, MZONE),
    (30655537, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, xyztg);
    e1.set_operation(duel, xyzop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TODECK + CATEGORY_REMOVE + CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, tg);
    e2.set_operation(duel, op);
    c.register_effect(duel, e2);
}

fn xyztg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && a.e.get_handler(duel).is_type(duel, TYPE_XYZ) && duel.get_field_group_count(1 - a.tp, LOCATION_DECK, 0) > 0)
}

fn xyzop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let g = duel.get_decktop_group(1 - a.tp, 1);
    if c.is_relate_to_effect(duel, a.e) && g.len() == 1 {
        duel.disable_shuffle_check(true);
        duel.overlay(c, g)?;
    }
    Ok(())
}

fn rescon(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    let count = |filter: fn(CardId, &Duel) -> bool| sg.filter_count(duel, filter, None);
    (
        count(CardId::is_monster) <= 1 && count(CardId::is_spell) <= 1 && count(CardId::is_trap) <= 1,
        false,
    )
}

fn available_types(duel: &Duel, c: CardId, tp: u8) -> u32 {
    let mut ty = 0;
    if c.is_able_to_remove(duel) {
        ty |= TYPE_MONSTER;
    }
    if duel.is_player_can_draw(tp, 1) {
        ty |= TYPE_SPELL;
    }
    if duel.is_existing_matching_card(aux::faceup_filter(CardId::is_able_to_deck), tp, 0, LOCATION_ONFIELD, 1, None) {
        ty |= TYPE_TRAP;
    }
    ty
}

fn tg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    let ty = available_types(duel, c, a.tp);
    Ok(chk == 0 && ty > 0 && c.get_overlay_group(duel).is_exists(duel, |c, duel| c.is_type(duel, ty), 1, None))
}

fn op(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let g = c.get_overlay_group(duel);
    let ty = available_types(duel, c, tp);
    if ty == 0 {
        return Ok(());
    }
    let mg = g.filter(duel, |c, duel| c.is_type(duel, ty), None);
    let sg = aux::select_unselect_group(duel, mg, 1, 3, &rescon, tp, HINTMSG_REMOVEXYZ)?;
    let lb = sg.into_iter().fold(0, |lb, tc| lb | tc.get_main_card_type(duel));
    duel.send_to_grave(sg, REASON_EFFECT)?;
    duel.raise_single_event(c, EVENT_DETACH_MATERIAL, Some(e), 0, 0, 0, 0)?;
    duel.break_effect()?;
    if lb & TYPE_MONSTER != 0 && c.is_relate_to_effect(duel, e) {
        let position = c.get_position(duel);
        duel.remove(c, position, REASON_EFFECT | REASON_TEMPORARY)?;
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e1.set_code(duel, EVENT_PHASE + PHASE_END);
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        e1.set_label_object_group(duel, Some(c.into()));
        e1.set_count_limit(duel, 1, 0);
        e1.set_operation(duel, retop);
        duel.register_effect(e1, tp);
    }
    if lb & TYPE_SPELL != 0 {
        duel.draw(tp, 1, REASON_EFFECT)?;
    }
    if lb & TYPE_TRAP != 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
        let g = duel.select_matching_card(tp, aux::faceup_filter(CardId::is_able_to_deck), tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
        if !g.is_empty() {
            duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKTOP, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn retop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(c) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) {
        duel.return_to_field(c)?;
    }
    Ok(())
}
