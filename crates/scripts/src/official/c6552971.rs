// A.I. Meet You
use engine::constants::MSG_DAMAGE;

use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND};

const ID: u32 = 6552971;

pub const LISTED_SERIES: &[u16] = &[SET_IGNISTER];

// In the hand with Dark Templar @Ignister (2300 ATK, DARK) in the Extra Deck to reveal, and DARK @Ignister monsters in the Deck
// to add.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (97383507, EXTRA), (3723262, DECK), (30118811, DECK), (3723262, DECK)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::Report(&Check::Message(MSG_DAMAGE))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_race(duel, RACE_CYBERSE)
        && c.get_attack(duel) == 2300
        && !c.is_public(duel)
        && duel.is_existing_matching_card(|tc, duel| filter(tc, duel, c), tp, LOCATION_DECK, 0, 1, None)
}

fn filter(c: CardId, duel: &Duel, rc: CardId) -> bool {
    c.is_set_card(duel, &[SET_IGNISTER]) && c.is_attribute(duel, rc.get_attribute(duel)) && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| cfilter(c, duel, tp), tp, LOCATION_HAND | LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
    let rc = duel.select_matching_card(
        tp,
        |c, duel| cfilter(c, duel, tp),
        tp,
        LOCATION_HAND | LOCATION_EXTRA,
        0,
        1,
        1,
        None,
    )?;
    if let Some(rc) = rc.get_first() {
        duel.confirm_cards(1 - tp, rc)?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let g = duel.select_matching_card(tp, |c, duel| filter(c, duel, rc), tp, LOCATION_DECK, 0, 1, 1, None)?;
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
        e1.set_operation(duel, regop);
        let code = rc.get_code(duel);
        e1.set_label(duel, i64::from(code));
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e1, tp);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e2.set_code(duel, EVENT_PHASE + PHASE_END);
        e2.set_count_limit(duel, 1, 0);
        e2.set_condition(duel, |duel, a| a.e.get_label_object(duel).is_some_and(|e1| e1.get_label(duel) != 0));
        e2.set_operation(duel, |duel, a| duel.damage(a.tp, 2300, REASON_EFFECT).map(drop));
        e2.set_reset(duel, RESET_PHASE | PHASE_END);
        e2.set_label_object(duel, Some(e1));
        duel.register_effect(e2, tp);
    }
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT + EFFECT_FLAG_OATH);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_target_range(duel, 1, 0);
    e3.set_value(duel, Value::EffectPlayer(aclimit));
    e3.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e3, tp);
    Ok(())
}

fn aclimit(duel: &Duel, _: EffectId, re: EffectId, _: u8) -> bool {
    re.is_monster_effect(duel) && !re.get_handler(duel).is_race(duel, RACE_CYBERSE)
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let code = a.e.get_label(duel);
    if code == 0 {
        return Ok(());
    }
    let tp = a.tp;
    if a.eg.unwrap_or_default().is_exists(
        duel,
        |c, duel| c.is_code(duel, code as u32) && c.is_summon_player(duel, tp),
        1,
        None,
    ) {
        a.e.set_label(duel, 0);
    }
    Ok(())
}
