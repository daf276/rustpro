// Shared Ride
use crate::prelude::*;
use crate::testing::{Check, HAND};

const ID: u32 = 1372887;

// In the hand, with Pair Bear Scare!! (also in the hand) going to the opponent's hand from the Graveyard after it resolves.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (21501961, HAND), (21501961, HAND)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_TO_HAND);
    e1.set_condition(duel, drcon1);
    e1.set_operation(duel, drop1);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_FIELD);
    e2.set_code(duel, EVENT_TO_HAND);
    e2.set_condition(duel, regcon);
    e2.set_operation(duel, regop);
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e2, tp);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_FIELD);
    e3.set_code(duel, EVENT_CHAIN_SOLVED);
    e3.set_condition(duel, |duel, a| duel.get_flag_effect(a.tp, ID) > 0);
    e3.set_operation(duel, drop2);
    e3.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e3, tp);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 1)));
    Ok(())
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_controler(duel, 1 - tp) && !c.is_reason(duel, REASON_DRAW) && c.is_previous_location(duel, LOCATION_DECK | LOCATION_GRAVE)
}

fn moved(duel: &Duel, a: &Args) -> bool {
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| cfilter(c, duel, a.tp), 1, None))
}

fn drcon1(duel: &Duel, a: &Args) -> bool {
    if !moved(duel, a) {
        return false;
    }
    unported!(duel, a.re.is_none());
    a.re.is_some_and(|re| !re.is_has_type(duel, EFFECT_TYPE_ACTIONS) || re.is_has_type(duel, EFFECT_TYPE_CONTINUOUS))
}

fn drop1(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.hint(HINT_CARD, 0, ID.into());
    duel.draw(a.tp, 1, REASON_EFFECT)?;
    Ok(())
}

fn regcon(duel: &Duel, a: &Args) -> bool {
    if !moved(duel, a) || duel.get_flag_effect(a.tp, ID) != 0 {
        return false;
    }
    unported!(duel, a.re.is_none());
    a.re.is_some_and(|re| re.is_has_type(duel, EFFECT_TYPE_ACTIONS) && !re.is_has_type(duel, EFFECT_TYPE_CONTINUOUS))
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.register_flag_effect(a.tp, ID, RESET_CHAIN, 0, 1);
    Ok(())
}

fn drop2(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.reset_flag_effect(a.tp, ID);
    duel.hint(HINT_CARD, 0, ID.into());
    duel.draw(a.tp, 1, REASON_EFFECT)?;
    Ok(())
}
