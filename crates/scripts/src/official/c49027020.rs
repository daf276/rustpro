// Traptrix Pudica
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND};

const ID: u32 = 49027020;

pub const LISTED_NAMES: &[u32] = &[12801833];
pub const LISTED_SERIES: &[u16] = &[SET_HOLE, SET_TRAP_HOLE];
// Copies in the hand and GY, Traptrip Garden to search and to Special Summon it, and Traptrix Dionaea to revive it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (12801833, DECK),
    (12801833, DECK),
    (12801833, 0),
    (45803070, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::Effect(immfilter));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_REMOVE + CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, rmtg);
    e3.set_operation(duel, rmop);
    c.register_effect(duel, e3);
}

pub fn immfilter(duel: &Duel, _: EffectId, te: EffectId) -> bool {
    let c = te.get_owner(duel);
    c.is_normal_trap(duel) && c.is_set_card(duel, &[SET_HOLE, SET_TRAP_HOLE])
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, 12801833) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn rmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_special_summoned(duel) && c.is_able_to_remove(duel)
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && rmfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(rmfilter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, rmfilter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, 0, 0);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, 1 - tp, LOCATION_REMOVED as i32);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
    {
        duel.remove(tc, POS_FACEUP, REASON_EFFECT)?;
    }
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    let (reset, mut reset_ct) = (RESET_PHASE | PHASE_STANDBY, 1);
    let mut turn_ct = 0;
    if duel.is_phase(PHASE_STANDBY) {
        reset_ct = 2;
        turn_ct = duel.get_turn_count();
    }
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, spcon);
    e1.set_operation(duel, spop);
    e1.set_label(duel, turn_ct.into());
    e1.set_reset_full(duel, reset, reset_ct);
    duel.register_effect(e1, tp);
    aux::register_client_hint_full(duel, c, 0, tp, 0, 1, Some(aux::stringid(ID, 2)), reset, reset_ct);
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_special_summoned(duel, e, 0, 1 - tp, false, false)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let (e, tp) = (a.e, a.tp);
    let label = e.get_label(duel);
    duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, 0, LOCATION_REMOVED, 1, None)
        && (label == 0 || label != i64::from(duel.get_turn_count()))
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, 0, LOCATION_REMOVED, None);
    if g.is_empty() || !duel.select_yes_no(1 - tp, aux::stringid(ID, 3))? {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let sg = g.select(duel, 1 - tp, 1, 1, None)?;
    if !sg.is_empty() {
        duel.special_summon(sg, 0, 1 - tp, 1 - tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
