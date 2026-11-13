// Bingo Card
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 99505609;
const CHOICE: u8 = 0;

// Copies in the hand and the Graveyard, with both sides' Main Monster Zones (Slot Machine) and Spell & Trap Zones (Trap Hole) full.
#[rustfmt::skip]
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND), (ID, HAND), (ID, GRAVE),
    (3797883, MZONE), (3797883, MZONE), (3797883, MZONE), (3797883, MZONE), (3797883, MZONE),
    (3797883, MZONE | OPPONENT), (3797883, MZONE | OPPONENT), (3797883, MZONE | OPPONENT),
    (3797883, MZONE | OPPONENT), (3797883, MZONE | OPPONENT),
    (4206964, SZONE), (4206964, SZONE), (4206964, SZONE), (4206964, SZONE), (4206964, SZONE),
    (4206964, SZONE | OPPONENT), (4206964, SZONE | OPPONENT), (4206964, SZONE | OPPONENT),
    (4206964, SZONE | OPPONENT), (4206964, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY + CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, efftg);
    e2.set_operation(duel, effop);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e2);
}

fn columnfilter(c: CardId, duel: &Duel) -> bool {
    c.get_column_group_count(duel) == if matches!(c.get_sequence(duel), 0 | 2 | 4) { 3 } else { 4 }
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(
            duel.is_existing_matching_card(columnfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None)
                && duel.is_player_can_draw(tp, 1)
                && duel.is_player_can_draw(1 - tp, 1),
        );
    }
    let g = duel.get_field_group(tp, LOCATION_ONFIELD, LOCATION_ONFIELD);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 4, tp, 0);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, PLAYER_ALL, 1);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_matching_card(tp, columnfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    let Some(sc) = g.first() else { return Ok(()) };
    duel.hint_selection(sc);
    let c = a.e.get_handler(duel);
    let mut g = sc.get_column_group(duel);
    g.insert(sc);
    if c.is_relate_to_effect(duel, a.e) {
        g.remove(c);
    }
    if duel.destroy(g, REASON_EFFECT)? > 0 {
        let turn_player = duel.get_turn_player();
        duel.break_effect()?;
        duel.draw(turn_player, 1, REASON_EFFECT)?;
        duel.draw(1 - turn_player, 1, REASON_EFFECT)?;
    }
    Ok(())
}

fn zones(duel: &Duel, player: u8) -> (CardSet, CardSet) {
    (
        duel.get_field_group(player, LOCATION_MMZONE, 0),
        duel.get_field_group(player, LOCATION_STZONE, 0),
    )
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let (own_mmzone, own_stzone) = zones(duel, tp);
    let b1 = (own_mmzone.len() == 5 || own_stzone.len() == 5) && duel.is_player_can_draw(tp, 2);
    let opp = 1 - tp;
    let (opp_mmzone, opp_stzone) = zones(duel, opp);
    let b2 = (opp_mmzone.len() == 5 || opp_stzone.len() == 5) && duel.is_player_can_draw(opp, 2);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let choice = duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?;
    e.set_chain_data(duel, CHOICE, choice.map_or(0, |choice| choice as i64));
    if choice == Some(1) {
        duel.set_operation_info(0, CATEGORY_DESTROY, Some(own_mmzone.union(own_stzone)), 5, tp, 0);
        duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 2);
    } else {
        duel.set_operation_info(0, CATEGORY_DESTROY, Some(opp_mmzone.union(opp_stzone)), 5, tp, 0);
        duel.set_operation_info(0, CATEGORY_DRAW, None, 0, opp, 2);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let player = if a.e.get_chain_data(duel, CHOICE) == Some(1) { tp } else { 1 - tp };
    let (mmzone, stzone) = zones(duel, player);
    let (b1, b2) = (mmzone.len() == 5, stzone.len() == 5);
    let op = if b1 && b2 {
        duel.select_effect(tp, &[(true, aux::stringid(ID, 4)), (true, aux::stringid(ID, 5))])?
    } else if b1 {
        Some(1)
    } else {
        b2.then_some(2)
    };
    if let Some(op) = op
        && duel.destroy(if op == 1 { mmzone } else { stzone }, REASON_EFFECT)? == 5
        && duel.is_player_can_draw(player, 0)
    {
        duel.break_effect()?;
        duel.draw(player, 2, REASON_EFFECT)?;
    }
    Ok(())
}
