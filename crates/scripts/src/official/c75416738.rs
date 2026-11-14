// Traptrix Arachnocampa
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 75416738;

pub const LISTED_SERIES: &[u16] = &[SET_TRAPTRIX, SET_HOLE];
// Copies in the hand, with Traptrix Myrmeleo on the field.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (91812341, MZONE), (91812341, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_MAIN_END);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, stock::special_summon_self_target_player_0);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, Value::Effect(immfilter));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e3.set_code(duel, EFFECT_INDESTRUCTABLE_COUNT);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, LOCATION_SZONE, 0);
    e3.set_target_card(duel, |duel, _, c| c.is_facedown(duel) && c.get_sequence(duel) < 5);
    e3.set_value(duel, Value::Reason(|_, _, _, r, _| r & REASON_EFFECT == REASON_EFFECT));
    c.register_effect(duel, e3);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let filter = aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_TRAPTRIX]));
    duel.is_main_phase() && duel.is_existing_matching_card(filter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if c.is_relate_to_effect(duel, a.e) {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_card(duel, |duel, _, c| {
        c.is_location(duel, LOCATION_EXTRA) && !c.is_race(duel, RACE_INSECT | RACE_PLANT)
    });
    e1.set_target_range(duel, 1, 0);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| !c.is_original_race(duel, RACE_INSECT | RACE_PLANT));
    Ok(())
}

fn immfilter(duel: &Duel, _: EffectId, te: EffectId) -> bool {
    let c = te.get_owner(duel);
    c.is_normal_trap(duel) && (c.is_set_card(duel, &[SET_HOLE]) || c.is_set_card(duel, &[SET_TRAP_HOLE]))
}
