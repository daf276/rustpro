// Exosister Betrayal
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, SZONE};

const ID: u32 = 52335937;

pub const LISTED_SERIES: &[u16] = &[SET_EXOSISTER];

// In the hand and face-up, with Exosister Sophia and Irene in the Deck to add, Irene on the field with Sophia in the hand to Normal
// Summon, and the opponent's Raigeki to send monsters to their Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (5352328, DECK),
    (79858629, DECK),
    (5352328, DECK),
    (79858629, MZONE),
    (5352328, HAND),
    (12580477, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_HANDES);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, nstg);
    e2.set_operation(duel, nsop);
    c.register_effect(duel, e2);
    let e3a = duel.create_effect(c);
    e3a.set_description(duel, aux::stringid(ID, 2));
    e3a.set_category(duel, CATEGORY_REMOVE);
    e3a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3a.set_property(duel, EFFECT_FLAG_DELAY);
    e3a.set_code(duel, EVENT_CUSTOM + ID);
    e3a.set_range(duel, LOCATION_SZONE);
    e3a.set_count_limit(duel, 1, (ID, 2));
    e3a.set_target(duel, rmtg);
    e3a.set_operation(duel, rmop);
    c.register_effect(duel, e3a);
    e3a.set_label_object_group(duel, Some(CardSet::default()));
    let e3b = duel.create_effect(c);
    e3b.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3b.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e3b.set_code(duel, EVENT_TO_GRAVE);
    e3b.set_range(duel, LOCATION_SZONE);
    e3b.set_label_object(duel, Some(e3a));
    e3b.set_operation(duel, regop);
    c.register_effect(duel, e3b);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_EXOSISTER]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 2, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 2, tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_HANDES, None, 1, tp, 1);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 2, 2, None)?;
    if g.len() == 2 && duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? > 0 {
        duel.confirm_cards(1 - tp, g)?;
        duel.shuffle_hand(tp)?;
        duel.break_effect()?;
        duel.discard_hand(tp, aux::TRUE, 1, 1, REASON_EFFECT | REASON_DISCARD)?;
    }
    Ok(())
}

fn tgfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel) && duel.is_existing_matching_card(|sc, duel| nsfilter(sc, duel, c), tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, None)
}

fn nsfilter(c: CardId, duel: &Duel, tc: CardId) -> bool {
    c.is_set_card(duel, &[SET_EXOSISTER]) && c.is_summonable(duel, true, None) && tc.lists_code(duel, c.get_code(duel))
}

fn nstg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && tgfilter(chkc, duel, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| tgfilter(c, duel, tp), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, |c, duel| tgfilter(c, duel, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_MZONE) as i32);
    Ok(false)
}

fn nsop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) && tc.is_faceup(duel) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SUMMON);
        let g = duel.select_matching_card(
            tp,
            |sc, duel| nsfilter(sc, duel, tc),
            tp,
            LOCATION_HAND | LOCATION_MZONE,
            0,
            1,
            1,
            None,
        )?;
        if let Some(sc) = g.get_first() {
            duel.summon(tp, sc, true, None)?;
        }
    }
    Ok(())
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tg =
        a.eg.unwrap_or_default()
            .filter(duel, |c, duel| c.is_monster(duel) && c.is_controler(duel, 1 - tp), None);
    if tg.is_empty() {
        return Ok(());
    }
    for tc in tg {
        tc.register_flag_effect(duel, ID, RESET_CHAIN, 0, 1);
    }
    let Some(e3a) = e.get_label_object(duel) else { return Ok(()) };
    let mut g = e3a.get_label_object_group(duel).unwrap_or_default();
    if duel.get_current_chain() == 0 {
        g.clear();
    }
    g.merge(tg);
    g.remove_matching(duel, |c, duel| c.get_flag_effect(duel, ID) == 0, None);
    e3a.set_label_object_group(duel, Some(g));
    duel.raise_single_event(e.get_handler(duel), EVENT_CUSTOM + ID, Some(e), 0, tp, tp, 0)
}

fn rmfilter(c: CardId, duel: &Duel, opp: u8) -> bool {
    c.is_controler(duel, opp) && c.is_location(duel, LOCATION_GRAVE) && c.is_able_to_remove_full(duel, Some(opp), POS_FACEUP, REASON_EFFECT)
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g =
        a.e.get_label_object_group(duel)
            .unwrap_or_default()
            .filter(duel, |c, duel| rmfilter(c, duel, 1 - tp), None);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_target_card(g);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(g), 1, 1 - tp, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    if g.is_empty() {
        return Ok(());
    }
    let opp = 1 - a.tp;
    duel.hint(HINT_SELECTMSG, opp, HINTMSG_REMOVE);
    let g = g.filter_select(
        duel,
        opp,
        |c, duel| c.is_able_to_remove_full(duel, Some(opp), POS_FACEUP, REASON_EFFECT),
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.remove_full(g, POS_FACEUP, REASON_EFFECT, PLAYER_NONE, Some(opp))?;
    }
    Ok(())
}
