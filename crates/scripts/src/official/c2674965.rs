// Argostars - Home Stadium
use super::c91284003::argosfilter as disconfilter;
use engine::constants::MSG_DAMAGE;

use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT, REMOVED, SZONE};

const ID: u32 = 2674965;

pub const LISTED_SERIES: &[u16] = &[SET_ARGOSTARS];
// Argostars cards to add back, Set Lightning Tydeus to summon, and the opponent's face-up monsters to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (91438674, GRAVE),
    (91438674, REMOVED),
    (65889305, SZONE),
    (65889305, SZONE),
    (65889305, SZONE),
    (91438674, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    // The damage of a Continuous Trap summoned from the Spell & Trap Zone forms no chain link.
    Check::Report(&Check::Message(MSG_DAMAGE)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_FZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::pay_lp::<1000>);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_condition(duel, damcon);
    e2.set_operation(duel, damop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DISABLE);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_CHAINING);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_CHAIN);
    e3.set_condition(duel, discon);
    e3.set_target(duel, distg);
    e3.set_operation(duel, disop);
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ARGOSTARS]) && c.is_faceup(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_GRAVE | LOCATION_REMOVED, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, aux::necro_valley_filter(thfilter), LOCATION_GRAVE | LOCATION_REMOVED)
}

fn damconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_continuous_trap(duel) && c.is_faceup(duel) && c.is_controler(duel, tp)
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| damconfilter(c, duel, a.tp), 1, None))
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.hint(HINT_CARD, 0, u64::from(ID));
    duel.damage(1 - a.tp, 500, REASON_EFFECT)?;
    Ok(())
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let ev = a.ev as u8;
    let trig_ctrl: u8 = duel.get_chain_info(ev, CHAININFO_TRIGGERING_CONTROLER);
    let trig_type: u32 = duel.get_chain_info(ev, CHAININFO_TRIGGERING_TYPE);
    let trig_loc: u32 = duel.get_chain_info(ev, CHAININFO_TRIGGERING_LOCATION);
    trig_ctrl == a.tp
        && trig_type & (TYPE_TRAP | TYPE_CONTINUOUS) > 0
        && trig_loc == LOCATION_MZONE
        && duel.is_existing_matching_card(disconfilter, a.tp, LOCATION_REMOVED, 0, 1, None)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_on_field(duel) && chkc.is_negatable(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_negatable, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
    let g = duel.select_target(tp, CardId::is_negatable, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DISABLE, g, 1, tp, 0);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tc = duel.get_first_target();
    unported!(duel, tc.is_none());
    if let Some(tc) = tc
        && tc.is_faceup(duel)
        && tc.is_relate_to_effect(duel, a.e)
    {
        tc.negate_effects(duel, a.e.get_handler(duel), RESET_PHASE | PHASE_END, true);
    }
    Ok(())
}
