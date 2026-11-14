// Megalith Anastasis
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 29876299;

pub const LISTED_SERIES: &[u16] = &[SET_MEGALITH];

// On the field, with Megalith Phul (Level 2) and Megalith Och (Level 4) and Megalith Bethor (Level 8) in the Deck to add, and Phul
// on the field to Ritual Summon Och with Kojikocy (Level 4), facing an opponent's monster to Tribute.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, HAND),
    (78990927, DECK),
    (36849933, DECK),
    (99628747, DECK),
    (99628747, DECK),
    (78990927, MZONE),
    (36849933, HAND),
    (1184620, HAND),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::discard);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, effcon);
    e2.set_target(duel, efftg);
    e2.set_operation(duel, effop);
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_level_below(duel, 4) || c.is_level_above(duel, 8)) && c.is_set_card(duel, &[SET_MEGALITH]) && c.is_able_to_hand(duel)
}

fn rescon(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    let res = sg.is_exists(duel, |c, duel| c.is_level_below(duel, 4), 1, None)
        && sg.is_exists(duel, |c, duel| c.is_level_above(duel, 8), 1, None);
    (res, false)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let g = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
        return Ok(aux::select_unselect_group_check(duel, g, 2, 2, &rescon));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 2, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
    if g.is_empty() {
        return Ok(());
    }
    let sg = aux::select_unselect_group(duel, g, 2, 2, &rescon, tp, HINTMSG_ATOHAND)?;
    if !sg.is_empty() {
        duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, sg)?;
    }
    Ok(())
}

fn effconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_summon_player(duel, tp) && c.is_ritual_summoned(duel) && c.is_set_card(duel, &[SET_MEGALITH]) && c.is_faceup(duel)
}

fn effcon(duel: &Duel, a: &Args) -> bool {
    a.eg.unwrap_or_default()
        .is_exists(duel, |c, duel| effconfilter(c, duel, a.tp), 1, None)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let b1 = !duel.has_flag_effect(tp, ID) && duel.is_player_can_draw(tp, 2);
    let b2 = !duel.has_flag_effect(tp, ID + 1)
        && duel.is_existing_matching_card(CardId::is_releasable_by_effect, tp, 0, LOCATION_MZONE, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
        e.set_category(duel, CATEGORY_DRAW + CATEGORY_HANDES);
        duel.set_operation_info(0, CATEGORY_DRAW, None, 1, tp, 2);
        duel.set_operation_info(0, CATEGORY_HANDES, None, 1, tp, 1);
    } else if op == Some(2) {
        duel.register_flag_effect(tp, ID + 1, RESET_PHASE | PHASE_END, 0, 1);
        e.set_category(duel, CATEGORY_RELEASE);
        duel.set_operation_info(0, CATEGORY_RELEASE, None, 1, 1 - tp, LOCATION_MZONE as i32);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    match a.e.get_label(duel) {
        1 => {
            if duel.draw(tp, 2, REASON_EFFECT)? == 2 {
                duel.shuffle_hand(tp)?;
                duel.break_effect()?;
                duel.discard_hand(tp, aux::TRUE, 1, 1, REASON_EFFECT | REASON_DISCARD)?;
            }
        }
        2 => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_RELEASE);
            let g = duel.select_matching_card(tp, CardId::is_releasable_by_effect, tp, 0, LOCATION_MZONE, 1, 1, None)?;
            if !g.is_empty() {
                duel.hint_selection(g);
                duel.release(g, REASON_EFFECT)?;
            }
        }
        _ => {}
    }
    Ok(())
}
