// Enneacraft Reverth
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT, PZONE};

const ID: u32 = 80015408;

pub const LISTED_SERIES: &[u16] = &[SET_ENNEACRAFT];

// In the hand and the GY, with Enneacraft monsters in the hand and on the field to shuffle, face-down to flip and in the hand
// to Set, and the opponent's Monster Reborn to answer.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (ID, GRAVE),
    (28454232, HAND),
    (28454232, HAND),
    (82359538, PZONE),
    (54842941, MZONE),
    (54842941, MZONE),
    (83764718, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TODECK + CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2a = duel.create_effect(c);
    e2a.set_description(duel, aux::stringid(ID, 1));
    e2a.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2a.set_property_full(duel, EFFECT_FLAG_DELAY, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e2a.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2a.set_range(duel, LOCATION_GRAVE);
    e2a.set_condition(duel, |_, a| a.rp == 1 - a.tp);
    e2a.set_cost(duel, Cost::self_banish);
    e2a.set_target(duel, efftg);
    e2a.set_operation(duel, effop);
    c.register_effect(duel, e2a);
    let e2b = e2a.clone_effect(duel);
    e2b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2b);
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ENNEACRAFT])
        && c.is_monster_card(duel)
        && (c.is_faceup(duel) || c.is_location(duel, LOCATION_HAND))
        && c.is_able_to_deck(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(
            duel.is_player_can_draw(tp, 0) && duel.is_existing_matching_card(tdfilter, tp, LOCATION_HAND | LOCATION_ONFIELD, 0, 1, None)
        );
    }
    duel.set_operation_info(0, CATEGORY_TODECK, None, 1, tp, (LOCATION_HAND | LOCATION_ONFIELD) as i32);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(tdfilter, tp, LOCATION_HAND | LOCATION_ONFIELD, 0, None);
    if g.is_empty() {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let sg = g.select(duel, tp, 1, g.len() as u8, None)?;
    if sg.is_empty() {
        return Ok(());
    }
    let (hg, fg) = sg.split(duel, |c, duel| c.is_location(duel, LOCATION_HAND), None);
    if !hg.is_empty() {
        duel.confirm_cards(1 - tp, hg)?;
    }
    if !fg.is_empty() {
        duel.hint_selection(fg);
    }
    if duel.send_to_deck(sg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? > 0 && duel.is_player_can_draw(tp, 0) {
        let og = duel
            .get_operated_group()
            .filter(duel, |c, duel| c.is_location(duel, LOCATION_DECK), None);
        if !og.is_empty() {
            if og.is_exists(duel, |c, duel| c.is_controler(duel, tp), 1, None) {
                duel.shuffle_deck(tp)?;
            }
            duel.break_effect()?;
            duel.draw(tp, og.len() as u32, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_ENNEACRAFT])
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEDOWN_DEFENSE, tp, 0xff)
}

fn posfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ENNEACRAFT]) && c.is_facedown(duel) && c.is_defense_pos(duel) && c.is_can_change_position(duel)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let b1 = duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, None);
    let b2 = duel.is_existing_matching_card(posfilter, tp, LOCATION_MZONE, 0, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        e.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_SET);
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    } else if op == Some(2) {
        e.set_category(duel, CATEGORY_POSITION);
        duel.set_operation_info(0, CATEGORY_POSITION, None, 1, tp, POS_FACEUP_DEFENSE as i32);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    match e.get_label(duel) {
        1 => {
            if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
                return Ok(());
            }
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
            if !g.is_empty() && duel.special_summon(g, 0, tp, tp, false, false, POS_FACEDOWN_DEFENSE)? > 0 {
                duel.confirm_cards(1 - tp, g)?;
            }
        }
        2 => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
            let g = duel.select_matching_card(tp, posfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
            if !g.is_empty() {
                duel.hint_selection(g);
                duel.change_position(g, POS_FACEUP_DEFENSE)?;
            }
        }
        _ => {}
    }
    Ok(())
}
