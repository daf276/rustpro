// Floowandereeze and the Unexplored Winds
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 55521751;

// Copies to activate, Level 8 Winged Beasts in hand to summon by sending a monster on the field of each player, and Robina in hand to
// return to the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, HAND),
    (61441708, HAND),
    (66431519, HAND),
    (18940725, HAND),
    (76812113, MZONE),
    (76812113, MZONE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_SUMMON_PROC);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_target_range(duel, LOCATION_HAND, 0);
    e2.set_condition(duel, otcon);
    e2.set_target_field_proc(duel, ottg, sumtg);
    e2.set_operation(duel, otop);
    e2.set_value(duel, SUMMON_TYPE_TRIBUTE as i32);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_SET_PROC);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_TODECK + CATEGORY_DRAW);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_count_limit(duel, 1, ID);
    e4.set_target(duel, drtg);
    e4.set_operation(duel, drop);
    c.register_effect(duel, e4);
}

fn tgfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_able_to_grave(duel) && !c.is_immune_to_effect(duel, e)
}

fn otcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let (tp, e) = (c.get_controler(duel), a.e);
    a.min <= 2
        && duel.get_location_count(tp, LOCATION_MZONE) > -1
        && duel.is_existing_matching_card(|c, duel| tgfilter(c, duel, e), tp, LOCATION_MZONE, 0, 1, None)
        && duel.is_existing_matching_card(|c, duel| tgfilter(c, duel, e), tp, 0, LOCATION_ONFIELD, 1, None)
}

fn ottg(duel: &Duel, _: EffectId, c: CardId) -> bool {
    let (mi, ma) = c.get_tribute_requirement(duel);
    mi <= 2 && ma >= 2
}

fn sumtg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (tp, e) = (a.tp, a.e);
    let mg1 = duel.get_matching_group(|c, duel| tgfilter(c, duel, e), tp, LOCATION_MZONE, 0, None);
    let mg2 = duel.get_matching_group(|c, duel| tgfilter(c, duel, e), tp, 0, LOCATION_ONFIELD, None);
    loop {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
        let mut g1 = mg1.select_full(duel, tp, 1, 1, true, None)?;
        if g1.is_empty() {
            return Ok(false);
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
        if let Some(tc) = mg2.select_unselect(duel, Some(g1), tp, false, false, 2, 2)?
            && mg2.is_contains(tc)
        {
            g1.insert(tc);
            e.set_label_object_group(duel, Some(g1));
            return Ok(true);
        }
    }
}

fn otop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(sg) = a.e.get_label_object_group(duel) {
        duel.send_to_grave(sg, REASON_EFFECT)?;
    }
    Ok(())
}

fn drfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_WINGEDBEAST) && !c.is_public(duel) && c.is_able_to_deck(duel)
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 1) && duel.is_existing_matching_card(drfilter, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TODECK, None, 1, tp, LOCATION_HAND as i32);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    Ok(false)
}

fn drop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_matching_card(tp, drfilter, tp, LOCATION_HAND, 0, 1, 2, None)?;
    if !g.is_empty() {
        duel.confirm_cards(1 - tp, g)?;
        let ct = duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKBOTTOM, REASON_EFFECT)?;
        if ct > 1 {
            duel.sort_deckbottom(tp, tp, ct as u16)?;
        }
        if ct as usize == g.len() {
            duel.break_effect()?;
            duel.draw(tp, ct as u32, REASON_EFFECT)?;
        }
    }
    Ok(())
}
