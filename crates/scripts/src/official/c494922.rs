// Superheavy Samurai Warlord Susanowo
use super::c36953371::Procedure;
use super::c82112494::gycon;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 494922;

pub const LISTED_SERIES: &[u16] = &[SET_SUPERHEAVY_SAMURAI];
// In the Extra Deck, Synchro Summoned from Motorbike (a Level 2 Tuner) and Big Benkei (Level 8), with Spells in the opponent's Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (83334932, MZONE),
    (83334932, MZONE),
    (3117804, MZONE),
    (3117804, MZONE),
    (53129443, GRAVE | OPPONENT),
    (12580477, GRAVE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_DEFENSE_ATTACK);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_hint_timing(duel, 0, TIMING_END_PHASE);
    e2.set_condition(duel, gycon);
    e2.set_target(duel, settg);
    e2.set_operation(duel, setop);
    c.register_effect(duel, e2);
}

fn setfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_spell_trap(duel)
        && c.is_ssetable_full(duel, true, None)
        && (c.is_type(duel, TYPE_FIELD) || duel.get_location_count(tp, LOCATION_SZONE) > 0)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_GRAVE) && setfilter(chkc, duel, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| setfilter(c, duel, tp), tp, 0, LOCATION_GRAVE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let g = duel.select_target(tp, |c, duel| setfilter(c, duel, tp), tp, 0, LOCATION_GRAVE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, g, 1, 0, 0);
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && (tc.is_type(duel, TYPE_FIELD) || duel.get_location_count(tp, LOCATION_SZONE) > 0) {
        duel.sset(tp, tc)?;
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_description(duel, 3300);
        e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
        e1.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
        e1.set_value(duel, LOCATION_REMOVED as i32);
        tc.register_effect_full(duel, e1, true);
    }
    Ok(())
}
