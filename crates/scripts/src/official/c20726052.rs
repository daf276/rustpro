// Maliss <C> GWC-06
use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE, REMOVED, SZONE};

const ID: u32 = 20726052;

pub const LISTED_SERIES: &[u16] = &[SET_MALISS];
// Set, with Maliss <P> March Hare and White Rabbit in the GY and banished, and Maliss <Q> Red Ransom on the field to banish
// for the turn it is Set and to gain LP.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (20938824, GRAVE),
    (69272449, REMOVED),
    (68059897, MZONE),
    (20938824, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_RECOVER);
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
        duel.is_existing_matching_card(|c, duel| spcostfilter(c, duel, tp), tp, LOCATION_MZONE, 0, 1, None)
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
        duel.get_location_count(a.e.get_handler_player(duel), LOCATION_MZONE) == 0
    });
    e3.set_value(
        duel,
        Value::Effect(|duel, e, te| te.get_code(duel) == EFFECT_TRAP_ACT_IN_SET_TURN && Some(te) != e.get_label_object(duel)),
    );
    e3.set_label_object(duel, Some(e2));
    c.register_effect(duel, e3);
}

fn spcostfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_set_card(duel, &[SET_MALISS])
        && c.is_able_to_remove_full(duel, Some(tp), POS_FACEUP, REASON_EFFECT)
        && duel.get_mzone_count(tp, Some(c.into())) > 0
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
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
        let g = duel.select_matching_card(tp, |c, duel| spcostfilter(c, duel, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?;
        duel.remove(g, POS_FACEUP, REASON_COST)?;
    }
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_MALISS]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let mut zone_chk = true;
        if !e.get_handler(duel).is_status(duel, STATUS_SET_TURN) || e.get_label(duel) != 100 {
            zone_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
        }
        e.set_label(duel, 0);
        let filter = |c, duel: &Duel| spfilter(c, duel, e, tp);
        return Ok(zone_chk && duel.is_existing_matching_card(filter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_GRAVE | LOCATION_REMOVED) as i32);
    duel.set_possible_operation_info(0, CATEGORY_RECOVER, None, 0, tp, 0);
    Ok(false)
}

fn linkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_MALISS]) && c.is_type(duel, TYPE_LINK)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, 1, None)?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    if duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)? == 0 {
        return Ok(());
    }
    let atk = tc.get_base_attack(duel);
    if atk > 0
        && duel.is_existing_matching_card(linkfilter, tp, LOCATION_MZONE, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.break_effect()?;
        duel.recover(tp, atk as u32, REASON_EFFECT)?;
    }
    Ok(())
}
