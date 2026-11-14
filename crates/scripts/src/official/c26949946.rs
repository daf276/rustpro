// Mecha Phantom Beast Jaculuslan
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 26949946;

pub const LISTED_SERIES: &[u16] = &[SET_MECHA_PHANTOM_BEAST];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_MECHA_PHANTOM_BEAST], Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp) && c.is_set_card_full(duel, &[SET_MECHA_PHANTOM_BEAST], Some(sc), sumtype, tp)
    }
}

pub const LISTED_NAMES: &[u32] = &[TOKEN_MECHA_PHANTOM_BEAST];

pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (72291078, MZONE),
    (41329458, MZONE),
    (31480215, HAND),
    (31480215, HAND),
    (5318639, DECK),
    (5318639, DECK),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_HANDES);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, hdcon);
    e1.set_cost(duel, hdcost);
    e1.set_target(duel, hdtg);
    e1.set_operation(duel, hdop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_target_card(duel, indtg);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_SET);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP);
    e4.set_code(duel, EVENT_DESTROYED);
    e4.set_condition(duel, setcon);
    e4.set_target(duel, settg);
    e4.set_operation(duel, setop);
    c.register_effect(duel, e4);
}

fn hdcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_synchro_summoned(duel)
}

fn is_token(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, TOKEN_MECHA_PHANTOM_BEAST)
}

fn hdcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(
            duel.check_release_group_cost(tp, is_token, (1, 1), false, None, None) && duel.get_field_group_count(tp, 0, LOCATION_HAND) > 0
        );
    }
    let ct = duel.get_field_group_count(tp, 0, LOCATION_HAND);
    let g = duel.select_release_group_cost(tp, is_token, (1, ct), false, None, None)?;
    e.set_label(duel, g.len() as i64);
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn hdtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let label = a.e.get_label(duel) as i32;
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, 1 - a.tp, label);
    Ok(false)
}

fn hdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_field_group(tp, 0, LOCATION_HAND);
    if !g.is_empty() {
        let count = e.get_label(duel) as usize;
        let sg = g.random_select(duel, 1 - tp, count);
        duel.send_to_grave(sg, REASON_DISCARD | REASON_EFFECT)?;
    }
    Ok(())
}

fn indtg(duel: &Duel, e: EffectId, c: CardId) -> bool {
    c.is_set_card(duel, &[SET_MECHA_PHANTOM_BEAST]) && c != e.get_handler(duel)
}

fn setcon(duel: &Duel, a: &Args) -> bool {
    a.rp != a.tp && a.e.get_handler(duel).is_previous_controler(duel, a.tp)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_quick_play_spell(duel) && c.is_ssetable(duel)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0 && duel.is_existing_matching_card(filter, tp, LOCATION_DECK, 0, 1, None));
    }
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        duel.sset(tp, tc)?;
    }
    Ok(())
}
