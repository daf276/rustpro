// Prohibitive Power Patron Purview - Vilaea
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};
use engine::constants::MSG_MOVE;

const ID: u32 = 99311889;

pub const LISTED_SERIES: &[u16] = &[SET_POWER_PATRON];

const NERVA: u32 = 53589300;
const JUPITER: u32 = 68231287;
const JUNORA: u32 = 5914858;

pub const LISTED_NAMES: &[u32] = &[NERVA, JUPITER, JUNORA];

// In hand to show the three Power Patron monsters in the Extra Deck, and in the Graveyard to replace the Deck banish cost of Nervedo
// the Shadebeast Power Patron.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (NERVA, EXTRA),
    (JUPITER, EXTRA),
    (JUNORA, EXTRA),
    (17473466, MZONE),
    (10266279, MZONE),
    (43871165, MZONE),
];
// The cost replacement is a REASON_COST banish of this card from the Graveyard, which no check isolates (it shows in the traces of
// scenarios 1, 3 and 8 to 12).
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::Message(MSG_MOVE)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EFFECT_COST_REPLACE);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_target_range(duel, 1, 0);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_able_to_remove_as_cost(duel));
    e2.set_value(duel, Value::CostReplace(repval));
    e2.set_operation(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        duel.remove(c, POS_FACEUP, REASON_COST).map(drop)
    });
    c.register_effect(duel, e2);
}

fn showfilter(c: CardId, duel: &Duel) -> bool {
    [NERVA, JUPITER, JUNORA].iter().any(|&code| c.is_code(duel, code)) && (c.is_faceup(duel) || !c.is_on_field(duel))
}

fn showable(duel: &Duel, tp: u8) -> CardSet {
    duel.get_matching_group(showfilter, tp, LOCATION_EXTRA | LOCATION_ONFIELD | LOCATION_GRAVE, 0, None)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(showable(duel, a.tp).get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) >= 3);
    }
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = showable(duel, tp);
    if g.len() >= 3 {
        let rg = aux::select_unselect_group(duel, g, 3, 3, &aux::dncheck, tp, HINTMSG_CONFIRM)?;
        let (fieldgraveg, extrag) = rg.split(duel, |c, duel| c.is_location(duel, LOCATION_ONFIELD | LOCATION_GRAVE), None);
        if !fieldgraveg.is_empty() {
            duel.hint_selection(fieldgraveg);
        }
        if !extrag.is_empty() {
            duel.confirm_cards(1 - tp, extrag)?;
        }
        if rg.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_EXTRA), 1, None) {
            duel.shuffle_extra(tp)?;
        }
        let c = e.get_handler(duel);
        aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 1)));
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e1.set_code(duel, EVENT_CHAINING);
        e1.set_operation(duel, inactop);
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e1, tp);
    }
    Ok(())
}

fn inactop(duel: &mut Duel, a: &Args) -> Script<()> {
    let ev = a.ev as u8;
    if a.ep == a.tp && chain::is_type(duel, ev, TYPE_MONSTER) && chain::is_setcode(duel, ev, &[SET_POWER_PATRON]) {
        duel.set_chain_limit(|_, _, rp, tp, _| rp == tp, 0);
    }
    Ok(())
}

fn repval(duel: &Duel, _: EffectId, call: CostReplaceCall) -> bool {
    let a = call.args;
    let c = a.e.get_handler(duel);
    c.is_set_card(duel, &[SET_POWER_PATRON]) && c.is_faceup(duel) && c.is_controler(duel, a.tp) && c.is_location(duel, LOCATION_MZONE)
}
