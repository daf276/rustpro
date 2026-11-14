// Centur-Ion Trudea
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 42493140;

pub const LISTED_SERIES: &[u16] = &[SET_CENTUR_ION];
pub const LISTED_NAMES: &[u32] = &[ID];

// On the field to place itself and Primera from the hand or Deck, and with a second Trudea in the hand and Deck.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, HAND), (ID, DECK), (15005145, DECK), (15005145, HAND)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, pltg);
    e1.set_operation(duel, plop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_LVCHANGE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_MAIN_END);
    e2.set_condition(duel, |duel, a| {
        duel.is_main_phase() && a.e.get_handler(duel).is_continuous_trap(duel)
    });
    e2.set_target(duel, stock::special_summon_self_target_player_0);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn plfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_CENTUR_ION]) && c.is_monster(duel) && !c.is_forbidden(duel) && !c.is_code(duel, ID)
}

fn pltg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(!a.e.get_handler(duel).is_forbidden(duel)
            && duel.get_location_count(tp, LOCATION_SZONE) >= 2
            && duel.is_existing_matching_card(plfilter, tp, LOCATION_HAND | LOCATION_DECK, 0, 1, None));
    }
    Ok(false)
}

fn move_to_stzone(duel: &mut Duel, c: CardId, hc: CardId, tp: u8) -> Script<bool> {
    let enable = c.is_monster_card(duel);
    if !duel.move_to_field(c, tp, tp, LOCATION_SZONE, POS_FACEUP, enable)? {
        return Ok(false);
    }
    let e1 = duel.create_effect(hc);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EFFECT_CHANGE_TYPE);
    e1.set_value(duel, (TYPE_TRAP | TYPE_CONTINUOUS) as i32);
    e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD & !RESET_TURN_SET));
    c.register_effect(duel, e1);
    Ok(true)
}

fn plop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| c.is_code(duel, ID));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    if c.is_forbidden(duel) || !c.is_relate_to_effect(duel, a.e) || duel.get_location_count(tp, LOCATION_SZONE) < 2 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    let g = duel.select_matching_card(tp, plfilter, tp, LOCATION_HAND | LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first()
        && move_to_stzone(duel, c, c, tp)?
    {
        move_to_stzone(duel, tc, c, tp)?;
    }
    Ok(())
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if c.is_relate_to_effect(duel, a.e)
        && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0
        && duel.select_yes_no(tp, aux::stringid(ID, 3))?
    {
        duel.break_effect()?;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_UPDATE_LEVEL);
        e1.set_value(duel, 4);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e1);
    }
    Ok(())
}
