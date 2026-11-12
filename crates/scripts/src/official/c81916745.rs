// Lycoris Lilyreaper
use super::c95040215::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 81916745;

// In the Extra Deck, Synchro Summoned from Water Spirit (Level 1 Tuner) and Prevent Rat (Level 4); Raigeki and Hinotama in hand.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, GRAVE),
    (487395, MZONE),
    (487395, MZONE),
    (549481, MZONE),
    (549481, MZONE),
    (12580477, HAND),
    (46130346, HAND),
    (46130346, HAND),
    (1929294, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
];

pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure<99>>(duel, c);
    let e1a = duel.create_effect(c);
    e1a.set_description(duel, aux::stringid(ID, 0));
    e1a.set_category(duel, CATEGORY_RECOVER);
    e1a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1a.set_property(duel, EFFECT_FLAG_DELAY);
    e1a.set_code(duel, EVENT_CUSTOM + ID);
    e1a.set_range(duel, LOCATION_MZONE);
    e1a.set_count_limit(duel, 1, ID);
    e1a.set_target(duel, lptg);
    e1a.set_operation(duel, lpop);
    e1a.set_label_object_group(duel, Some(CardSet::default()));
    c.register_effect(duel, e1a);
    let e1b = duel.create_effect(c);
    e1b.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1b.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1b.set_code(duel, EVENT_TO_GRAVE);
    e1b.set_range(duel, LOCATION_MZONE);
    e1b.set_label_object(duel, Some(e1a));
    e1b.set_operation(duel, regop);
    c.register_effect(duel, e1b);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_DAMAGE);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |_, a| a.r & REASON_EFFECT == REASON_EFFECT && a.ep == 1 - a.tp);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, tgtg);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
}

fn regfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_controler(duel, 1 - tp) && c.is_previous_location(duel, LOCATION_MZONE) && c.get_attack(duel) > 0 && c.is_monster(duel)
}

fn lptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g =
        a.e.get_label_object_group(duel)
            .unwrap_or_default()
            .filter(duel, |c, duel| regfilter(c, duel, tp), None);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_target_card(g);
    duel.register_flag_effect(tp, ID, RESET_CHAIN, 0, 1);
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, tp, 0);
    Ok(false)
}

fn lpop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    if g.is_empty() {
        return Ok(());
    }
    let mut sc = g.get_first();
    if g.len() > 1 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SELECT);
        sc = g.select(duel, tp, 1, 1, None)?.get_first();
    }
    if let Some(sc) = sc {
        duel.hint_selection(sc);
        let atk = sc.get_attack(duel);
        duel.recover(tp, atk as u32, REASON_EFFECT)?;
    }
    Ok(())
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tg = a.eg.unwrap_or_default().filter(duel, |c, duel| regfilter(c, duel, tp), None);
    if tg.is_empty() {
        return Ok(());
    }
    for tc in tg {
        tc.register_flag_effect(duel, ID, RESET_CHAIN, 0, 1);
    }
    let Some(e1a) = e.get_label_object(duel) else { return Ok(()) };
    let mut g = e1a.get_label_object_group(duel).unwrap_or_default();
    if duel.get_current_chain() == 0 {
        g.clear();
    }
    g = g.union(tg).filter(duel, |c, duel| c.get_flag_effect(duel, ID) != 0, None);
    e1a.set_label_object_group(duel, Some(g));
    if duel.get_flag_effect(tp, ID) == 0 {
        duel.register_flag_effect(tp, ID, RESET_CHAIN, 0, 1);
        duel.raise_single_event(e.get_handler(duel), EVENT_CUSTOM + ID, Some(e), 0, tp, tp, 0)?;
    }
    Ok(())
}

fn tgfilter(c: CardId, duel: &Duel, lp: i32) -> bool {
    c.is_attack_below(duel, lp) && c.is_faceup(duel) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let lp = (duel.get_lp(tp) - duel.get_lp(1 - tp)).abs();
        return Ok(duel.is_existing_matching_card(|c, duel| tgfilter(c, duel, lp), tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, PLAYER_EITHER, LOCATION_MZONE as i32);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let lp = (duel.get_lp(tp) - duel.get_lp(1 - tp)).abs();
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, |c, duel| tgfilter(c, duel, lp), tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.send_to_grave(g, REASON_EFFECT)?;
    }
    Ok(())
}
