// Angelechy Problem
use crate::prelude::*;
use crate::testing::{Check, HAND};

const ID: u32 = 17782288;

pub const LISTED_SERIES: &[u16] = &[SET_ANGELECHY];
// A Field Spell for Angelechy Opening to e4 and Angelechy Destrier, Level 7, and Angelechy Shatranga, to be placed.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND)];
pub const CHECKS: &[Check] = &[Check::Resolves];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_FZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, |duel, a, chk| {
        Cost::discard_full(duel, a, chk, CardId::is_spell_trap, false, (1, 1))
    });
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOEXTRA + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_count_limit_full(duel, 1, 0, EFFECT_COUNT_CODE_CHAIN);
    e2.set_condition(duel, tdcon);
    e2.set_target(duel, tdtg);
    e2.set_operation(duel, tdop);
    c.register_effect(duel, e2);
}

fn plfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ANGELECHY]) && !c.is_forbidden(duel)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level(duel, &[2])
        && c.is_set_card(duel, &[SET_ANGELECHY])
        && duel.get_location_count_from_ex(tp, tp, None, c) > 0
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && duel.is_existing_matching_card(plfilter, tp, LOCATION_EXTRA, 0, 1, Some(c.into()))
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if g.is_empty() || duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)? == 0 || duel.get_location_count(tp, LOCATION_SZONE) == 0
    {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    let g = duel.select_matching_card(tp, plfilter, tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first()
        && duel.move_to_field(sc, tp, tp, LOCATION_SZONE, POS_FACEUP, true)?
    {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_CHANGE_TYPE);
        e1.set_value(duel, (TYPE_SPELL | TYPE_CONTINUOUS) as i32);
        e1.set_reset(duel, RESET_EVENT | (RESETS_STANDARD & !RESET_TURN_SET));
        sc.register_effect(duel, e1);
    }
    Ok(())
}

fn tdconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_previous_set_card(duel, &[SET_ANGELECHY])
        && c.is_monster_card(duel)
        && c.is_previous_controler(duel, tp)
        && c.is_previous_position(duel, POS_FACEUP)
        && c.is_previous_location(duel, LOCATION_ONFIELD)
        && c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
}

fn tdcon(duel: &Duel, a: &Args) -> bool {
    a.eg.unwrap_or_default()
        .is_exists(duel, |c, duel| tdconfilter(c, duel, a.tp), 1, None)
}

fn tdfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_ANGELECHY])
        && c.is_monster_card(duel)
        && c.is_owner(duel, tp)
        && c.is_faceup(duel)
        && c.is_able_to_extra(duel)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| tdfilter(c, duel, tp), tp, LOCATION_STZONE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOEXTRA, None, 1, tp, LOCATION_STZONE as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_matching_card(tp, |c, duel| tdfilter(c, duel, tp), tp, LOCATION_STZONE, 0, 1, 1, None)?;
    let Some(sc) = g.get_first() else { return Ok(()) };
    duel.hint_selection(sc);
    if duel.send_to_deck(sc, PLAYER_NONE, SEQ_DECKBOTTOM, REASON_EFFECT)? > 0
        && sc.is_location(duel, LOCATION_EXTRA)
        && duel.get_location_count_from_ex(tp, tp, None, sc) > 0
        && sc.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.break_effect()?;
        duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
