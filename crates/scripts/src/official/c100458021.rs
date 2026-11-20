// Raise Moon Starter Squeeze
use super::c100458016::{drcost, track_searches};
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 100458021;
const CHOICE: u8 = 0;

// Copies in the hand to discard, and Raigeki for the opponent's Sangans, whose searches the first effect draws for.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (12580477, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_cost(duel, |duel, a, chk| {
        Cost::and(duel, a, chk, &[Cost::self_discard, drcost::<ID, 1, 0>])
    });
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
    track_searches::<ID, 0>(duel, c);
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let option_1 = !duel.has_flag_effect(tp, ID + 100);
    let option_2 = !duel.has_flag_effect(tp, ID + 200) && duel.is_player_can_draw(tp, 1);
    if chk == 0 {
        return Ok(option_1 || option_2);
    }
    let choice = duel.select_effect(tp, &[(option_1, aux::stringid(ID, 2)), (option_2, aux::stringid(ID, 3))])?;
    e.set_chain_data(duel, CHOICE, choice.map_or(0, |choice| choice as i64));
    if choice == Some(1) {
        duel.register_flag_effect(tp, ID + 100, RESET_PHASE | PHASE_END, 0, 1);
        duel.set_possible_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    } else if choice == Some(2) {
        duel.register_flag_effect(tp, ID + 200, RESET_PHASE | PHASE_END, 0, 1);
        duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let choice = a.e.get_chain_data(duel, CHOICE).unwrap_or(0);
    if choice == 1 {
        let c = a.e.get_handler(duel);
        let e1 = duel.create_effect(c);
        e1.set_description(duel, aux::stringid(ID, 4));
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
        e1.set_target_range(duel, 1, 0);
        e1.set_target_card(duel, |duel, _, c| {
            c.is_location(duel, LOCATION_EXTRA) && !(c.is_rank(duel, &[7]) && c.is_xyz_monster(duel))
        });
        e1.set_reset_full(duel, RESET_PHASE | PHASE_END, 2);
        duel.register_effect(e1, tp);
        aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 5)));
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e2.set_code(duel, EVENT_TO_HAND);
        e2.set_condition(duel, drcon);
        e2.set_operation(duel, drop);
        e2.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e2, tp);
    } else if choice == 2 {
        duel.draw(tp, 1, REASON_EFFECT)?;
    }
    Ok(())
}

fn drconfilter(c: CardId, duel: &Duel, opp: u8) -> bool {
    c.is_controler(duel, opp) && !c.is_reason(duel, REASON_DRAW)
}

fn drcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.unwrap_or_default()
        .is_exists(duel, |c, duel| drconfilter(c, duel, 1 - tp), 1, None)
}

// e2 keeps e1, an upvalue of its Lua closure, as its label object.
fn drop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if !duel.is_chain_solving() {
        duel.hint(HINT_CARD, 1 - tp, ID.into());
        duel.draw(tp, 1, REASON_EFFECT)?;
        return Ok(());
    }
    if let Some(eff) = e.get_label_object(duel).filter(|eff| !eff.is_deleted(duel)) {
        let label = eff.get_label(duel);
        eff.set_label(duel, label + 1);
        return Ok(());
    }
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_CHAIN_SOLVED);
    e1.set_operation(duel, chainsolvedop);
    e1.set_label(duel, 1);
    e1.set_label_object(duel, Some(e));
    e1.set_reset(duel, RESET_CHAIN);
    duel.register_effect(e1, tp);
    e.set_label_object(duel, Some(e1));
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_CHAIN_SOLVED);
    e2.set_operation(duel, |duel, a| {
        if let Some(e1) = a.e.get_label_object(duel) {
            if let Some(e) = e1.get_label_object(duel) {
                e.set_label_object(duel, None);
            }
            e1.reset(duel);
        }
        Ok(())
    });
    e2.set_label_object(duel, Some(e1));
    e2.set_reset(duel, RESET_CHAIN);
    duel.register_effect(e2, tp);
    Ok(())
}

fn chainsolvedop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    duel.hint(HINT_CARD, 1 - a.tp, ID.into());
    duel.draw(a.tp, e.get_label(duel) as u32, REASON_EFFECT)?;
    e.reset(duel);
    if let Some(eff) = e.get_label_object(duel) {
        eff.set_label_object(duel, None);
    }
    Ok(())
}
