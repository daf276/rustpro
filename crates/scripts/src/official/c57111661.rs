// Maliss <C> TB-11
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE};

const ID: u32 = 57111661;

pub const LISTED_SERIES: &[u16] = &[SET_MALISS];
// With Maliss <P> March Hare in the Deck to Special Summon and on the field to banish for the turn it is Set.
pub const SUPPORT: &[(u32, u16)] = &[(ID, 0), (20938824, DECK), (20938824, DECK), (96676583, DECK), (20938824, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_cost(duel, spcost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e2.set_code(duel, EFFECT_TRAP_ACT_IN_SET_TURN);
    e2.set_value(duel, Value::Activation(|duel, e, _| e.set_label(duel, 1)));
    e2.set_condition(duel, |duel, a| {
        let tp = a.e.get_handler_player(duel);
        let loc = summon_locations(duel, tp);
        duel.is_existing_matching_card(|c, duel| spcostfilter(c, duel, loc, a.e, tp), tp, LOCATION_MZONE, 0, 1, None)
    });
    c.register_effect(duel, e2);
    e1.set_label_object(duel, Some(e2));
    // Workaround: unaffected by other EFFECT_TRAP_ACT_IN_SET_TURN effects while the field is full
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(
        duel,
        EFFECT_FLAG_SET_AVAILABLE + EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_IGNORE_IMMUNE + EFFECT_FLAG_UNCOPYABLE,
    );
    e3.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e3.set_condition(duel, |duel, a| {
        let tp = a.e.get_handler_player(duel);
        let loc = summon_locations(duel, tp);
        let e = a.e;
        !duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp, None, false), tp, loc, 0, 1, None)
    });
    e3.set_value(
        duel,
        Value::Effect(|duel, e, te| te.get_code(duel) == EFFECT_TRAP_ACT_IN_SET_TURN && Some(te) != e.get_label_object(duel)),
    );
    e3.set_label_object(duel, Some(e2));
    c.register_effect(duel, e3);
}

fn summon_locations(duel: &Duel, tp: u8) -> u32 {
    if duel.get_field_group_count(tp, 0, LOCATION_ONFIELD) >= 3 {
        LOCATION_DECK | LOCATION_EXTRA
    } else {
        LOCATION_DECK
    }
}

fn spcostfilter(c: CardId, duel: &Duel, loc: u32, e: EffectId, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_set_card(duel, &[SET_MALISS])
        && c.is_able_to_remove_full(duel, Some(tp), POS_FACEUP, REASON_EFFECT)
        && duel.is_existing_matching_card(|sc, duel| spfilter(sc, duel, e, tp, Some(c), false), tp, loc, 0, 1, None)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mc: Option<CardId>, ignore_zone_chk: bool) -> bool {
    if !(c.is_monster(duel) && c.is_set_card(duel, &[SET_MALISS]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)) {
        return false;
    }
    if mc.is_some() || !ignore_zone_chk {
        if c.is_location(duel, LOCATION_EXTRA) {
            return c.is_type(duel, TYPE_LINK) && duel.get_location_count_from_ex(tp, tp, mc.map(Into::into), c) > 0;
        }
        return duel.get_mzone_count(tp, mc.map(Into::into)) > 0;
    }
    true
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    e.set_label(duel, 100);
    let Some(label_obj) = e.get_label_object(duel) else {
        return Ok(chk == 0);
    };
    if chk == 0 {
        label_obj.set_label(duel, 0);
        return Ok(true);
    }
    if label_obj.get_label(duel) > 0 {
        label_obj.set_label(duel, 0);
        let loc = summon_locations(duel, tp);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
        let g = duel.select_matching_card(tp, |c, duel| spcostfilter(c, duel, loc, e, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?;
        duel.remove(g, POS_FACEUP, REASON_COST)?;
    }
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let ignore_zone_chk = e.get_handler(duel).is_status(duel, STATUS_SET_TURN) && e.get_label(duel) == 100;
        e.set_label(duel, 0);
        let loc = summon_locations(duel, tp);
        return Ok(duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp, None, ignore_zone_chk), tp, loc, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_DECK | LOCATION_EXTRA) as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let loc = summon_locations(duel, tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp, None, false), tp, loc, 0, 1, 1, None)?;
    if g.is_empty() || duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)? <= 0 {
        return Ok(());
    }
    let Some(tc) = g.get_first() else { return Ok(()) };
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, 3206);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    tc.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_description(duel, 3302);
    e2.set_code(duel, EFFECT_CANNOT_TRIGGER);
    tc.register_effect(duel, e2);
    Ok(())
}
