// Rahamu, Envoy of the Sacred Tome
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 53904087;

// In the Extra Deck, Link Summoned with two Sangan (Effect Monsters), with Shapesnatch and Megasonic Eye (Level 5 DARK Normal Monsters) in the hand to Normal Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (26202165, MZONE),
    (26202165, MZONE),
    (4035199, HAND),
    (7562372, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_NO_TURN_RESET);
    e1.set_code(duel, EFFECT_SUMMON_PROC);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_HAND, 0);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, ntcon);
    e1.set_target_card(duel, |duel, _, c| c.is_level_above(duel, 5));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, _| duel.is_main_phase());
    e2.set_target(duel, nstg);
    e2.set_operation(duel, nsop);
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_TODECK + CATEGORY_DRAW);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3.set_code(duel, EVENT_PHASE + PHASE_END);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e3.set_target(duel, drtg);
    e3.set_operation(duel, drop);
    c.register_effect(duel, e3);
}

fn ntcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    a.e.get_handler(duel).is_link_summoned(duel) && a.min == 0 && duel.get_location_count(c.get_controler(duel), LOCATION_MZONE) > 0
}

fn nsfilter(c: CardId, duel: &Duel) -> bool {
    c.is_summonable(duel, true, None) && c.is_level_above(duel, 5) && c.is_attribute(duel, ATTRIBUTE_DARK)
}

fn nstg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, nsfilter, LOCATION_HAND | LOCATION_MZONE, CATEGORY_SUMMON)
}

fn nsop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SUMMON);
    if let Some(sc) = duel
        .select_matching_card(tp, nsfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, 1, None)?
        .get_first()
    {
        duel.summon(tp, sc, true, None)?;
    }
    Ok(())
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_able_to_deck(duel)
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(tdfilter, tp, LOCATION_HAND, 0, 1, None) && duel.is_player_can_draw(tp, 0));
    }
    duel.set_target_player(tp);
    duel.set_operation_info(0, CATEGORY_TODECK, None, 1, tp, LOCATION_HAND as i32);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    Ok(false)
}

fn drop(duel: &mut Duel, _: &Args) -> Script<()> {
    let target_player: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let max_count = duel.get_matching_group_count(tdfilter, target_player, LOCATION_HAND, 0, None);
    if max_count == 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, target_player, HINTMSG_TODECK);
    let g = duel.select_matching_card(target_player, tdfilter, target_player, LOCATION_HAND, 0, 1, max_count as u8, None)?;
    if !g.is_empty() {
        duel.confirm_cards(1 - target_player, g)?;
        let draw_count = duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKBOTTOM, REASON_EFFECT)?;
        if draw_count > 0 {
            if draw_count > 1 {
                duel.sort_deckbottom(target_player, target_player, draw_count as u16)?;
            }
            duel.break_effect()?;
            duel.draw(target_player, draw_count as u32, REASON_EFFECT)?;
        }
    }
    Ok(())
}
