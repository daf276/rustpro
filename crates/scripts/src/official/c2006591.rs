// Sinful Spoils Awakening
use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE, OPPONENT, SZONE};

use super::c62173132::settg;

const ID: u32 = 2006591;

// Set and in the GY, with Diabellze the White Witch on the field and three Sangan of the opponent's.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (ID, GRAVE),
    (60145298, MZONE),
    (60145298, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, ID);
    e1.set_hint_timing(
        duel,
        0,
        TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E | TIMING_BATTLE_PHASE,
    );
    e1.set_condition(duel, thcon);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_LEAVE_GRAVE + CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_hint_timing(
        duel,
        TIMING_END_PHASE,
        TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMING_BATTLE_END | TIMINGS_CHECK_MONSTER_E,
    );
    e2.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(illusionconfilter, a.tp, LOCATION_MZONE, 0, 1, None)
    });
    e2.set_target(duel, settg);
    e2.set_operation(duel, setop);
    c.register_effect(duel, e2);
}

fn illusionconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level_above(duel, 5) && c.is_race(duel, RACE_ILLUSION) && c.is_faceup(duel)
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(illusionconfilter, a.tp, LOCATION_MZONE, 0, 1, None)
        && duel.get_field_group_count(a.tp, 0, LOCATION_MZONE) >= 3
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_on_field(duel) && chkc.is_able_to_hand(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_able_to_hand, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_target(tp, CardId::is_able_to_hand, tp, 0, LOCATION_ONFIELD, 1, 3, None)?;
    let count = g.map_or(0, |g| g.len());
    duel.set_operation_info(0, CATEGORY_TOHAND, g, count as u8, tp, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(g) = duel.get_target_cards(a.e)
        && !g.is_empty()
    {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && c.is_ssetable(duel) && duel.sset(a.tp, c)? > 0 {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3300);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
        e1.set_value(duel, LOCATION_REMOVED as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
        c.register_effect(duel, e1);
    }
    Ok(())
}
