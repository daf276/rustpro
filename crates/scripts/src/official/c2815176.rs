// Zebufera, Vaalmonican Hallow Heathen
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT, PZONE, REMOVED};
use engine::{Operation, Target};

const ID: u32 = 2815176;

pub const LISTED_SERIES: &[u16] = &[SET_VAALMONICA];
pub const COUNTER_LIST: &[u16] = &[COUNTER_RESONANCE];

pub const LISTED_NAMES: &[u32] = &[ID];

// A Link Monster's Link Summon procedure, shared with Duralume: `RACE` is the Race of the Pendulum Monster that needs 3 Resonance Counters.
pub struct Procedure<const RACE: u64>;

impl<const RACE: u64> LinkProcedure for Procedure<RACE> {
    const MIN: usize = 1;
    const MAX: Option<usize> = Some(1);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }

    fn spcon(duel: &Duel, _: EffectId, tp: u8) -> bool {
        splimit::<RACE>(duel, tp, SUMMON_TYPE_LINK)
    }
}

pub fn splimit<const RACE: u64>(duel: &Duel, sp: u8, st: u32) -> bool {
    st & SUMMON_TYPE_LINK != SUMMON_TYPE_LINK
        || duel
            .get_matching_group(
                aux::faceup_filter(|c, duel| c.is_original_race(duel, RACE)),
                sp,
                LOCATION_PZONE,
                0,
                None,
            )
            .iter()
            .map(|c| u32::from(c.get_counter(duel, COUNTER_RESONANCE)))
            .sum::<u32>()
            >= 3
}

// On the field, with Angello and Dimonno in the Pendulum Zone and Vaalmonica Spells banished and in the Graveyard to apply; destroyed by the opponent's Raigeki.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (3048768, PZONE),
    (30432463, PZONE),
    (65496951, GRAVE),
    (42193638, REMOVED),
    (5605529, GRAVE),
    (78598237, REMOVED),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure<RACE_FIEND>>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_SPSUMMON_CONDITION);
    e0.set_value(
        duel,
        Value::SpSummonCondition(|duel, _, _, sp, st, _, _| splimit::<RACE_FIEND>(duel, sp, st)),
    );
    c.register_effect(duel, e0);
    c.set_sp_summon_once(duel, ID);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EFFECT_DESTROY_REPLACE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target(duel, reptg);
    e1.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(repfilter(c, duel, e.get_handler_player(duel)))),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e2.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e2.set_target(duel, cptg);
    e2.set_operation(duel, cpop);
    c.register_effect(duel, e2);
}

fn repfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_controler(duel, tp)
        && c.is_on_field(duel)
        && c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
        && !c.is_reason(duel, REASON_REPLACE)
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(a.eg.unwrap_or_default().is_exists(duel, |c, duel| repfilter(c, duel, tp), 1, None)
            && duel.is_can_remove_counter(tp, 1, 0, COUNTER_RESONANCE, 3, REASON_EFFECT));
    }
    if duel.select_effect_yes_no_full(tp, c, 96)? {
        duel.remove_counter(tp, 1, 0, COUNTER_RESONANCE, 3, REASON_EFFECT)
    } else {
        Ok(false)
    }
}

fn cpfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_VAALMONICA])
        && (c.is_normal_spell(duel) || c.is_normal_trap(duel))
        && c.is_faceup(duel)
        && c.check_activate_effect(duel, false, true).is_some()
}

fn cptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE | LOCATION_REMOVED) && chkc.is_controler(duel, tp) && cpfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(cpfilter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let Some(tc) = duel
        .select_target(tp, cpfilter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, 1, None)?
        .and_then(|g| g.get_first())
    else {
        return Ok(false);
    };
    let Some(eff) = tc.check_activate_effect(duel, false, true) else {
        return Ok(false);
    };
    e.set_property(duel, EFFECT_FLAG_CARD_TARGET | eff.get_property_full(duel).0);
    match eff.get_target(duel) {
        Target::None => {}
        Target::Action(tg) => {
            tg(duel, a, 1, None)?;
        }
        _ => unported!(duel, true),
    }
    duel.clear_operation_info(0);
    Ok(false)
}

fn cpop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let Some(op) = tc.check_activate_effect(duel, false, true).and_then(|eff| eff.get_operation(duel)) else {
        return Ok(());
    };
    unported!(duel, !matches!(op, Operation::Event(_) | Operation::Promptless(_)));
    if let Operation::Event(op) | Operation::Promptless(op) = op {
        op(duel, a)?;
    }
    Ok(())
}
