// Absolute King Back Jack
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};
use engine::constants::MSG_SORT_CARD;

const ID: u32 = 60990740;

// In the Graveyard, and on the field to be destroyed by the opponent's Raigeki, with Normal Traps (Trap Hole, Waboku,
// Raigeki Break) in the Deck to excavate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, GRAVE),
    (ID, MZONE),
    (12580477, HAND | OPPONENT),
    (4206964, DECK),
    (4206964, DECK),
    (12607053, DECK),
    (12607053, DECK),
    (4178474, DECK),
    (4178474, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::Message(MSG_SORT_CARD)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_hint_timing(duel, 0, TIMING_END_PHASE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e1.set_cost(duel, Cost::self_banish);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, sdtg);
    e2.set_operation(duel, sdop);
    c.register_effect(duel, e2);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_discard_deck(tp, 1)
            && duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.can_player_set_spell_trap(tp));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !duel.is_player_can_discard_deck(tp, 1) || !duel.can_player_set_spell_trap(tp) {
        return Ok(());
    }
    duel.confirm_decktop(tp, 1)?;
    let Some(tc) = duel.get_decktop_group(tp, 1).get_first() else {
        return Ok(());
    };
    duel.disable_shuffle_check(true);
    if tc.is_normal_trap(duel) && tc.is_ssetable(duel) {
        duel.sset(tp, tc)?;
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_TRAP_ACT_IN_SET_TURN);
        e1.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        e1.set_description(duel, aux::stringid(ID, 0));
        tc.register_effect(duel, e1);
    } else {
        duel.send_to_grave(tc, REASON_EFFECT | REASON_EXCAVATE)?;
    }
    Ok(())
}

fn sdtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && duel.get_field_group_count(a.tp, LOCATION_DECK, 0) > 2)
}

fn sdop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.sort_decktop(a.tp, a.tp, 3)
}
