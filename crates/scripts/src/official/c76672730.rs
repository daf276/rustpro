// Dark World Archives
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, SZONE};

const ID: u32 = 76672730;

pub const LISTED_SERIES: &[u16] = &[SET_DARK_WORLD];
// Beiige, Vanguard of Dark World in the hand to discard and on the field to power up; Zure, Knight of Dark World (a Fiend
// Normal Monster) in the hand for the second discard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, HAND),
    (33731070, HAND),
    (33731070, HAND),
    (33731070, MZONE),
    (7459013, HAND),
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
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_HANDES + CATEGORY_ATKCHANGE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, atktg);
    e2.set_operation(duel, atkop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_HANDES + CATEGORY_DRAW);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP);
    e3.set_code(duel, EVENT_DISCARD);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, drwcond);
    e3.set_target(duel, drwtg);
    e3.set_operation(duel, drwop);
    c.register_effect(duel, e3);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DARK_WORLD]) && c.is_monster(duel) && c.is_discardable(duel)
}

fn faceup_dark_world(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_DARK_WORLD])
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(cfilter, tp, LOCATION_HAND, 0, 1, None)
            && duel.is_existing_matching_card(faceup_dark_world, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, tp, 1);
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DISCARD);
    let dg = duel.select_matching_card(tp, cfilter, tp, LOCATION_HAND, 0, 1, 1, None)?;
    if dg.is_empty() || duel.send_to_grave(dg, REASON_EFFECT | REASON_DISCARD)? <= 0 {
        return Ok(());
    }
    let og = duel.get_operated_group();
    let sg = duel.get_matching_group(faceup_dark_world, tp, LOCATION_MZONE, 0, None);
    if sg.is_empty() {
        return Ok(());
    }
    let atk = og.get_first().map_or(0, |oc| oc.get_level(duel)) * 100;
    let handler = a.e.get_handler(duel);
    for tc in sg {
        let e1 = duel.create_effect(handler);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, atk as i32);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
    }
    Ok(())
}

fn dfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_controler(duel, tp)
        && c.is_previous_controler(duel, tp)
        && c.is_original_race(duel, RACE_FIEND)
        && c.is_reason(duel, REASON_EFFECT)
}

fn drwcond(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| dfilter(c, duel, tp), 1, None))
        && (a.re.is_some_and(|re| re.get_handler(duel).is_set_card(duel, &[SET_DARK_WORLD])) || a.rp == 1 - tp)
}

fn drwtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, None) && duel.is_player_can_draw(tp, 2));
    }
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, tp, 1);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 2);
    Ok(false)
}

fn drwop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DISCARD);
    let g = duel.select_matching_card(tp, CardId::is_discardable, tp, LOCATION_HAND, 0, 1, 1, None)?;
    if !g.is_empty() && duel.send_to_grave(g, REASON_EFFECT | REASON_DISCARD)? > 0 {
        duel.break_effect()?;
        duel.draw(tp, 2, REASON_EFFECT)?;
    }
    Ok(())
}
