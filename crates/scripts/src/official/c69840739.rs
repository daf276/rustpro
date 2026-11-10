// Artifact Durendal
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 69840739;

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(5);
    const CT: usize = 2;
}

// In the Extra Deck, Xyz Summoned with Level 5 Shapesnatches, and a Spell and a Trap of the opponent's to chain to or destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (4035199, MZONE),
    (4035199, MZONE),
    (4035199, MZONE),
    (4035199, MZONE),
    (53129443, HAND | OPPONENT),
    (4206964, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let event = duel.check_event_full(EVENT_CHAINING);
    let b1 = event.is_some_and(|event| {
        let Some(re) = event.reason_effect else { return false };
        let location: u32 = duel.get_chain_info(event.event_value as u8, CHAININFO_TRIGGERING_LOCATION);
        duel.is_existing_matching_card(CardId::is_spell_trap, event.reason_player, 0, LOCATION_ONFIELD, 1, None)
            && ((re.is_monster_effect(duel) && location == LOCATION_MZONE)
                || (re.get_handler(duel).is_normal_spell_trap(duel) && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)))
    });
    let b2 = (duel.is_player_can_draw(tp, 0) || duel.get_field_group_count(tp, LOCATION_HAND, 0) == 0)
        && (duel.is_player_can_draw(1 - tp, 0) || duel.get_field_group_count(tp, 0, LOCATION_HAND) == 0)
        && duel.is_existing_matching_card(CardId::is_able_to_deck, tp, LOCATION_HAND, LOCATION_HAND, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = if b1 {
        duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?
            .unwrap_or(0)
    } else {
        2
    };
    e.set_label(duel, op as i64);
    if op == 1 {
        e.set_category(duel, 0);
        duel.set_target_param(event.map_or(0, |event| event.event_value));
    } else if op == 2 {
        e.set_category(duel, CATEGORY_TODECK + CATEGORY_DRAW);
        if !b1 {
            duel.hint(HINT_OPSELECTED, 1 - tp, aux::stringid(ID, 2));
        }
        duel.set_operation_info(0, CATEGORY_TODECK, None, 1, PLAYER_ALL, LOCATION_HAND as i32);
        duel.set_operation_info(0, CATEGORY_DRAW, None, 0, PLAYER_ALL, 1);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let op = a.e.get_label(duel);
    if op == 1 {
        let event_value: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
        duel.change_target_card(event_value as u8, CardSet::default());
        duel.change_chain_operation(event_value as u8, repop);
    } else if op == 2 {
        let g = duel.get_field_group(tp, LOCATION_HAND, LOCATION_HAND);
        if duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKTOP, REASON_EFFECT)? > 0 {
            let og = duel
                .get_operated_group()
                .filter(duel, |c, duel| c.is_location(duel, LOCATION_DECK), None);
            let turn_player = duel.get_turn_player();
            let non_turn_player = 1 - turn_player;
            for player in [turn_player, non_turn_player] {
                if og.is_exists(duel, |c, duel| c.is_controler(duel, player), 1, None) {
                    duel.shuffle_deck(player)?;
                }
            }
            let ct1 = og.filter_count(duel, |c, duel| c.is_previous_controler(duel, turn_player), None);
            let ct2 = og.filter_count(duel, |c, duel| c.is_previous_controler(duel, non_turn_player), None);
            duel.break_effect()?;
            if ct1 > 0 {
                duel.draw(turn_player, ct1 as u32, REASON_EFFECT)?;
            }
            if ct2 > 0 {
                duel.draw(non_turn_player, ct2 as u32, REASON_EFFECT)?;
            }
        }
    }
    Ok(())
}

fn repop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if c.is_normal_spell_trap(duel) {
        c.cancel_to_grave_full(duel, false);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_matching_card(tp, CardId::is_spell_trap, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
