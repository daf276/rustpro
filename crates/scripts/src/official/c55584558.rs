// Purrely Delicious Memory
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 55584558;

pub const LISTED_SERIES: &[u16] = &[SET_PURRELY];
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (26202165, MZONE), (26202165, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_HANDES + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_hint_timing(duel, TIMING_MAIN_END, TIMING_MAIN_END);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_XMATERIAL);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_set_card(duel, &[SET_PURRELY]));
    e2.set_value(
        duel,
        Value::Card(|duel, e, _| e.get_handler(duel).get_overlay_count(duel) as i32 * 300),
    );
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e3);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(aux::TRUE, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.set_possible_operation_info(0, CATEGORY_HANDES, None, 0, tp, 1);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_PURRELY]) && c.is_level(duel, &[1]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_APPLYTO);
    let g = duel.select_matching_card(tp, aux::TRUE, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    duel.hint_selection_full(tc, true);
    let extraproperty = if tc.is_position(duel, POS_FACEDOWN) {
        EFFECT_FLAG_SET_AVAILABLE
    } else {
        0
    };
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, 3000);
    e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT + extraproperty);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_reset_full(duel, RESETS_STANDARD_PHASE_END, 2);
    e1.set_value(duel, 1);
    tc.register_effect(duel, e1);
    if duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, None)
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 1))?
    {
        duel.break_effect()?;
        if duel.discard_hand(tp, CardId::is_discardable, 1, 1, REASON_EFFECT | REASON_DISCARD)? > 0 {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
            if !g.is_empty() {
                duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
            }
        }
    }
    Ok(())
}
